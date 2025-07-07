use axum::{
    extract::FromRequestParts,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::common::error::{ErrorResponse, ErrorTypes};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: u32,
    pub exp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct AuthHeader {
    pub claims: Claims,
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct RefreshHeader {
    pub claims: Claims,
    pub token: String,
}

impl<S: std::marker::Sync> FromRequestParts<S> for AuthHeader {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|s| s.split_whitespace().last())
            .ok_or("Missing header")
            .map_err(|why| {
                tracing::error!("{}", why);
                (
                    StatusCode::BAD_REQUEST,
                    axum::Json(ErrorResponse::new(
                        ErrorTypes::NoAuthHeader,
                        "No auth header",
                    )),
                )
                    .into_response()
            })?;

        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(std::env::var("SECRET_WORD_JWT").unwrap().as_ref()),
            &Validation::default(),
        )
        .map_err(|err| {
            tracing::error!("Could not validate: {}", err);
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(ErrorResponse::new(
                    ErrorTypes::JwtTokenExpired,
                    "Token update requested",
                )),
            )
                .into_response()
        })?
        .claims;

        Ok(AuthHeader {
            claims,
            token: token.to_owned(),
        })
    }
}

impl<S: std::marker::Sync> FromRequestParts<S> for RefreshHeader {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|s| s.split_whitespace().last())
            .ok_or("Missing header")
            .map_err(|why| {
                tracing::error!("{}", why);
                (
                    StatusCode::BAD_REQUEST,
                    axum::Json(ErrorResponse::new(
                        ErrorTypes::NoAuthHeader,
                        "No auth header",
                    )),
                )
                    .into_response()
            })?;

        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(std::env::var("SECRET_WORD_REFRESH").unwrap().as_ref()),
            &Validation::default(),
        )
        .map_err(|err| {
            tracing::error!("Could not validate: {}", err);
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(ErrorResponse::new(
                    ErrorTypes::JwtTokenExpired,
                    "Token expired",
                )),
            )
                .into_response()
        })?
        .claims;

        Ok(RefreshHeader {
            claims,
            token: token.to_owned(),
        })
    }
}

pub fn make_jwt_token(user_id: u32) -> String {
    let claims = Claims {
        id: user_id,
        exp: (Utc::now() + Duration::hours(1)).timestamp(),
    };
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(std::env::var("SECRET_WORD_JWT").unwrap().as_ref()),
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
        &EncodingKey::from_secret(std::env::var("SECRET_WORD_REFRESH").unwrap().as_ref()),
    )
    .unwrap()
}

pub fn verify_jwt_token(token: &str) -> anyhow::Result<u32> {
    let validation = Validation::default();

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(std::env::var("SECRET_WORD_JWT").unwrap().as_ref()),
        &validation,
    )?;
    Ok(claims.claims.id)
}
