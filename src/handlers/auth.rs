use axum::{
    extract::{Query, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::{controllers::{tokendb, userdb}, crypt::{self, token}};

use super::types::{AuthError, AuthErrors};

#[derive(Deserialize, Serialize)]
pub struct UserRegister {
    username: String,
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserLogin {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct TokensResponse {
    jwt_token: String,
    refresh_token: String,
}

pub async fn register(
    State(pool): State<MySqlPool>,
    Json(user_data): Json<UserRegister>,
) -> Result<Response, Response> {
    if (user_data.username.len() > 32 || user_data.username.is_empty())
        || (user_data.password.is_empty()
            || user_data.password.len() < 4
            || user_data.password.len() > 64)
        || (user_data.email.is_empty())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AuthError::new(AuthErrors::BadData, "Provided data is bad")),
        )
            .into_response());
    }

    let hashed_password = crypt::password::hash_password(&user_data.password);

    match userdb::create_user(
        &pool,
        &user_data.username,
        &user_data.email,
        &hashed_password,
    )
    .await
    {
        Ok(id) => {
            let jwt_token = crypt::token::make_jwt_token(id);
            let refresh_token = crypt::token::make_refresh_token(id);

            tokendb::create_token(&pool, id, &refresh_token).await.unwrap();

            let resp = TokensResponse {
                jwt_token,
                refresh_token,
            };
            return Ok((StatusCode::CREATED, Json(resp)).into_response());
        }
        Err(why) => {
            eprintln!("Error registering: {}", why);
            return Err((
                StatusCode::CONFLICT,
                Json(AuthError::new(
                    AuthErrors::UserAlreadyExists,
                    "User is already registered",
                )),
            )
                .into_response());
        }
    }
}

pub async fn login(
    State(pool): State<MySqlPool>,
    Json(user_data): Json<UserLogin>,
) -> Result<Response, Response> {
    if (user_data.password.is_empty()
        || user_data.password.len() < 4
        || user_data.password.len() > 64)
        || (user_data.email.is_empty())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AuthError::new(AuthErrors::BadData, "Provided data is bad")),
        )
            .into_response());
    }

    let user_id = match userdb::id_by_email(&pool, &user_data.email).await {
        Ok(id) => id,
        Err(why) => {
            eprintln!("{}", why);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(AuthError::new(
                    AuthErrors::UserNotExists,
                    "User does not exist, please register",
                )),
            )
                .into_response());
        }
    };
    let user_password_hash = userdb::get_password_hash(&pool, user_id).await.unwrap(); // User exists 100% by now so does the password hash

    match crypt::password::verify_password(&user_data.password, &user_password_hash) {
        Ok(()) => {
            let jwt_token = crypt::token::make_jwt_token(user_id);
            let refresh_token = crypt::token::make_refresh_token(user_id);

            tokendb::create_token(&pool, user_id, &refresh_token).await.unwrap();

            let resp = TokensResponse {
                jwt_token,
                refresh_token,
            };
            return Ok((StatusCode::OK, Json(resp)).into_response());
        }
        Err(why) => {
            eprintln!("Error verify: {}", why);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthError::new(
                    AuthErrors::InvalidCreds,
                    "Invalid credentials",
                )),
            )
                .into_response());
        }
    }
}

pub async fn token(State(pool): State<MySqlPool>, headers: HeaderMap) -> Result<Response, Response> {
    if let Some(authorization) = headers.get(AUTHORIZATION) {
        let refresh_tkn = authorization
            .to_str()
            .unwrap()
            .split_whitespace()
            .nth(1)
            .unwrap_or("");

        if !tokendb::token_exists(&pool, refresh_tkn).await {
            return Err((
                StatusCode::FORBIDDEN,
                Json(AuthError::new(
                    AuthErrors::RefreshTokenExpired,
                    "Please, log in again",
                )),
            )
                .into_response());
        } 
        match crypt::token::verify_refresh_token(refresh_tkn) {
            Ok(id) => {
                let jwt_token = crypt::token::make_jwt_token(id);
                return Ok((StatusCode::OK, jwt_token.to_string()).into_response());
            }
            Err(why) => {

                eprintln!("Error verify refresh: {}", why);
                tokendb::delete_token(&pool, refresh_tkn).await.unwrap();
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(AuthError::new(
                        AuthErrors::RefreshTokenExpired,
                        "Please, log in again",
                    )),
                )
                    .into_response());
            }
        }
    }
    return Err((StatusCode::BAD_REQUEST, "No token in headers".to_string()).into_response());
}

#[derive(Serialize, Deserialize)]
pub struct QueryValidate {
    token: String,
}

pub async fn validate(Query(data): Query<QueryValidate>) -> Result<Response, Response> {
    match token::verify_jwt_token(&data.token) {
        Ok(_) => return Ok((StatusCode::OK, "Token was verified").into_response()),
        Err(why) => {
            eprintln!("Error {}", why);
            return Err((StatusCode::UNAUTHORIZED, "Token was not verified").into_response());
        }
    };
}

#[derive(Serialize, Deserialize)]
pub struct LogoutBody {
    refresh_token: String
}

pub async fn logout(State(pool): State<MySqlPool>, Json(data) : Json<LogoutBody>) -> Result<Response, Response> {
    if let Err(why) = tokendb::delete_token(&pool, &data.refresh_token).await {
        eprintln!("Err deleting rtoken: {}", why);
    }
    Ok((StatusCode::OK, "Error deleting").into_response())
}
