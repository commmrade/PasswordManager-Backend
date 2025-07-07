use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::{
    common::error::{AppError, ErrorTypes},
    controllers,
    crypt::{
        self,
        token::{self, RefreshHeader},
    },
    error_response,
};

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
) -> Result<Response, AppError> {
    if (user_data.username.len() > 32 || user_data.username.is_empty())
        || (user_data.password.is_empty()
            || user_data.password.len() < 4
            || user_data.password.len() > 64)
        || (user_data.email.is_empty())
    {
        return Ok(error_response!(
            StatusCode::BAD_REQUEST,
            ErrorTypes::BadData,
            "Provided data is bad"
        ));
    }

    let hashed_password = crypt::password::hash_password(&user_data.password);

    match controllers::users::create_user(
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
            controllers::tokens::create_token(&pool, id, &refresh_token).await?;

            let resp = TokensResponse {
                jwt_token,
                refresh_token,
            };
            return Ok((StatusCode::CREATED, Json(resp)).into_response());
        }
        Err(why) => {
            tracing::error!("Could not create user: {}", why);

            return Ok(error_response!(
                StatusCode::CONFLICT,
                ErrorTypes::UserAlreadyExists,
                "User is already registered: {}",
                why
            ));
        }
    }
}

pub async fn login(
    State(pool): State<MySqlPool>,
    Json(user_data): Json<UserLogin>,
) -> Result<Response, AppError> {
    if (user_data.password.is_empty()
        || user_data.password.len() < 4
        || user_data.password.len() > 64)
        || (user_data.email.is_empty())
    {
        return Ok(error_response!(
            StatusCode::BAD_REQUEST,
            ErrorTypes::BadData,
            "Provided data is bad"
        ));
    }

    let user_id = controllers::users::id_by_email(&pool, &user_data.email).await?;

    let user_password_hash = controllers::users::get_password_hash(&pool, user_id).await?;

    if let Err(why) = crypt::password::verify_password(&user_data.password, &user_password_hash) {
        tracing::error!("Could not verify user: {}", why);

        return Ok(error_response!(
            StatusCode::UNAUTHORIZED,
            ErrorTypes::InvalidCreds,
            "Invalid credentials: {}",
            why
        ));
    }

    let jwt_token = crypt::token::make_jwt_token(user_id);
    let refresh_token = crypt::token::make_refresh_token(user_id);
    controllers::tokens::create_token(&pool, user_id, &refresh_token).await?;

    let resp = TokensResponse {
        jwt_token,
        refresh_token,
    };
    return Ok((StatusCode::OK, Json(resp)).into_response());
}

pub async fn update_jwt_token(
    State(pool): State<MySqlPool>,
    refresh_header: RefreshHeader,
) -> Result<Response, AppError> {
    if let Err(why) = controllers::tokens::token_exists(&pool, &refresh_header.token).await {
        return Ok(error_response!(
            StatusCode::FORBIDDEN,
            ErrorTypes::RefreshTokenExpired,
            "Refresh token expired: {}",
            why
        ));
    }

    let jwt_token = crypt::token::make_jwt_token(refresh_header.claims.id);
    return Ok((StatusCode::OK, jwt_token.to_string()).into_response());
}

#[derive(Serialize, Deserialize)]
pub struct QueryValidate {
    token: String,
}

pub async fn validate(Query(data): Query<QueryValidate>) -> Result<Response, Response> {
    if let Err(_) = token::verify_jwt_token(&data.token) {
        return Ok(error_response!(
            StatusCode::UNAUTHORIZED,
            ErrorTypes::JwtTokenExpired,
            "Token was not verified",
        ));
    }
    Ok((StatusCode::OK).into_response())
}

#[derive(Serialize, Deserialize)]
pub struct LogoutBody {
    refresh_token: String,
}

pub async fn logout(
    State(pool): State<MySqlPool>,
    Json(data): Json<LogoutBody>,
) -> Result<Response, Response> {
    if let Err(why) = controllers::tokens::delete_token(&pool, &data.refresh_token).await {
        tracing::error!("Err deleting rtoken: {}", why);
    }
    Ok((StatusCode::OK).into_response())
}
