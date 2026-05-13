use anyhow::anyhow;
use openidconnect::core::{
    CoreAuthenticationFlow, CoreClient, CoreProviderMetadata,
};
use openidconnect::{AccessToken, reqwest};
use openidconnect::{
    AccessTokenHash, AuthorizationCode, ClientId, CsrfToken, IssuerUrl, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, RedirectUrl, Scope, TokenResponse,
};
use owo_colors::OwoColorize as _;
use std::io::Write;
use std::net::TcpListener;

use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};

use crate::output::Output;
use crate::util;

pub fn authorize(output: Arc<Mutex<Output>>) -> anyhow::Result<AccessToken> {
    let http_client = reqwest::blocking::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    // Use OpenID Connect Discovery to fetch the provider metadata.
    let provider_metadata = CoreProviderMetadata::discover(
        &IssuerUrl::new("http://localhost:8080/realms/my-realm".to_string())?,
        &http_client,
    )?;

    // Create an OpenID Connect client by specifying the client ID, client secret, authorization URL
    // and token URL.
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new("my-client".to_string()),
        None,
    )
    // Set the URL the user will be redirected to after the authorization process.
    .set_redirect_uri(RedirectUrl::new(
        "http://localhost:8070/callback".to_string(),
    )?);

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) =
        PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, _csrf_token, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // Set the desired scopes.
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    // This is the URL you should redirect the user to, in order to trigger the authorization
    // process.
    webbrowser::open(&auth_url.to_string())?;

    util::info(
        &output,
        format!(
            "Waiting for OAuth callback on {} ...",
            "http://localhost:8070/callback".yellow().bold()
        ),
    );

    let listener = TcpListener::bind("127.0.0.1:8070")?;

    let mut code: Option<String> = None;
    let mut state: Option<String> = None;

    if let Ok((mut stream, _addr)) = listener.accept() {
        let mut reader = BufReader::new(stream.try_clone()?);

        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;

        // Example:
        // GET /callback?code=abc&state=xyz HTTP/1.1
        let parts: Vec<&str> = request_line.split_whitespace().collect();

        if parts.len() >= 2 {
            let path = parts[1];

            if let Some(query_start) = path.find('?') {
                let query = &path[query_start + 1..];

                for (k, v) in url::form_urlencoded::parse(query.as_bytes()) {
                    match k.as_ref() {
                        "code" => code = Some(v.to_string()),
                        "state" => state = Some(v.to_string()),
                        _ => {}
                    }
                }
            }
        }

        let response_body = "Login complete. You can close this tab.";

        let response = format!(
            "HTTP/1.1 200 OK\r\n\
                Content-Type: text/html; charset=utf-8\r\n\
                Content-Length: {}\r\n\
                Connection: close\r\n\
                \r\n\
                {}",
            response_body.len(),
            response_body
        );

        stream.write_all(response.as_bytes())?;
        stream.flush()?;
    }

    let code =
        code.ok_or_else(|| anyhow::anyhow!("Missing authorization code"))?;
    let _state = state.ok_or_else(|| anyhow::anyhow!("Missing state"))?;

    let auth_code = AuthorizationCode::new(code);

    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    // Now you can exchange it for an access token and ID token.
    let token_response = client
        .exchange_code(auth_code)?
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
        .request(&http_client)?;

    // Extract the ID token claims after verifying its authenticity and nonce.
    let id_token = token_response
        .id_token()
        .ok_or_else(|| anyhow!("Server did not return an ID token"))?;
    let id_token_verifier = client.id_token_verifier();
    let claims = id_token.claims(&id_token_verifier, &nonce)?;

    // Verify the access token hash to ensure that the access token hasn't been substituted for
    // another user's.
    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash = AccessTokenHash::from_token(
            token_response.access_token(),
            id_token.signing_alg()?,
            id_token.signing_key(&id_token_verifier)?,
        )?;
        if actual_access_token_hash != *expected_access_token_hash {
            return Err(anyhow!("Invalid access token"));
        }
    }

    // The authenticated user's identity is now available. See the IdTokenClaims struct for a
    // complete listing of the available claims.
    util::info(
        &output,
        format!(
            "User {} with e-mail address {} has authenticated successfully",
            claims.subject().to_string().yellow().bold().to_string(),
            claims
                .email()
                .map(|email| email.as_str())
                .unwrap_or("<not provided>")
                .to_string()
                .yellow()
                .bold()
                .to_string(),
        ),
    );

    Ok(token_response.access_token().to_owned())
}
