use std::collections::BTreeMap;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const SECRET_WORD: &str = "vladivostok"; // TODO: make it env

#[derive(Serialize, Deserialize)]
struct Claims {
    id: u32,
    exp: i64,
}

pub fn make_jwt_token(user_id: u32) -> String {
    let claims = Claims {
        id: user_id,
        exp: (Utc::now() + Duration::hours(3)).timestamp(),
    };
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET_WORD.as_ref()),
    )
    .unwrap()
}

pub fn make_refresh_token(user_id: u32) -> String {
    let claims = Claims {
        id: user_id,
        exp: (Utc::now() + Duration::days(7)).timestamp(),
    };
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET_WORD.as_ref()),
    )
    .unwrap()
}

pub fn verify_refresh_token(token: &str) -> anyhow::Result<u32> {
    let validation = Validation::default();
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_WORD.as_ref()),
        &validation,
    )?;
    Ok(claims.claims.id)
}

pub fn verify_jwt_token(token: &str) -> anyhow::Result<u32> {
    let validation = Validation::default();
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_WORD.as_ref()),
        &validation,
    )?;
    Ok(claims.claims.id)
}
