use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::{controllers::userdb, crypt};

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
struct TokenResponse {
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

            let resp = TokenResponse {
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

            let resp = TokenResponse {
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
