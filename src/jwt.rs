#![allow(unused)]

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::sync::RwLock;

#[derive(Debug, Deserialize)]
struct Claims {
    iss: String,
    aud: Audience,
    exp: usize,
    sub: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Audience {
    Single(String),
    Multiple(Vec<String>),
}

impl Audience {
    fn contains(&self, expected: &str) -> bool {
        match self {
            Audience::Single(a) => a == expected,
            Audience::Multiple(v) => v.iter().any(|a| a == expected),
        }
    }
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

    let n = key["n"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing n"))?;
    let e = key["e"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing e"))?;

    Ok(DecodingKey::from_rsa_components(n, e)?)
}

pub fn verify_jwt(token: &str) -> anyhow::Result<()> {
    let jwks = fetch_jwks()?;
    let key = get_rsa_key(token, &jwks)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[ISSUER]);
    validation.set_audience(&["account"]);

    decode::<Claims>(token, &key, &validation)?;

    Ok(())
}
