#![allow(unused)]

use jsonwebtoken::{
    Algorithm, DecodingKey, TokenData, Validation, decode, decode_header,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::sync::RwLock;

#[derive(Debug, Deserialize)]
struct Claims {
    iss: String,
    exp: usize,
    sub: String,
    preferred_username: String,
}

const ISSUER: &str = "http://localhost:8080/realms/my-realm";
const CLIENT_ID: &str = "my-client";

static JWKS: Lazy<RwLock<Option<jsonwebtoken::DecodingKey>>> =
    Lazy::new(|| RwLock::new(None));

fn fetch_jwks() -> anyhow::Result<serde_json::Value> {
    let url = format!("{}/protocol/openid-connect/certs", ISSUER);

    Ok(reqwest::blocking::get(url)?.json()?)
}

fn get_rsa_key(
    token: &str,
    jwks: &serde_json::Value,
) -> anyhow::Result<DecodingKey> {
    let header = decode_header(token)?;
    let kid = header.kid.ok_or_else(|| anyhow::anyhow!("Missing kid"))?;

    let keys = jwks["keys"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Invalid JWKS"))?;

    let key = keys
        .iter()
        .find(|k| k["kid"].as_str() == Some(&kid))
        .ok_or_else(|| anyhow::anyhow!("No matching key for kid"))?;

    let modulus = key["n"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing n"))?;
    let exponent = key["e"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing e"))?;

    Ok(DecodingKey::from_rsa_components(modulus, exponent)?)
}

pub fn verify_jwt(token: &str) -> anyhow::Result<String> {
    let jwks = fetch_jwks()?;
    let key = get_rsa_key(token, &jwks)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[ISSUER]);
    validation.set_audience(&["account"]);

    let token_data = decode::<Claims>(token, &key, &validation)?;
    Ok(token_data.claims.preferred_username)
}
