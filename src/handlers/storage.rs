use std::{fs::OpenOptions, io::Write};

use axum::{
    body::Body,
    extract::{multipart, Multipart, Query, State},
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const STORAGE_FILENAME: &str = "pmanager.pm";
const PASSWORD_FILENAME: &str = "password.txt";

use crate::crypt::token;

#[derive(Serialize, Deserialize)]
pub struct UploadReq {
    password: String,
}

#[axum::debug_handler]
pub async fn upload(headers: HeaderMap, mut multipart: Multipart) -> Result<Response, Response> {
    let empty = HeaderValue::from_str("").unwrap();
    let token = headers
        .get("Authorization")
        .unwrap_or(&empty)
        .to_str()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .expect("Token is not set");
    let password_of_storage = headers
        .get("Password")
        .unwrap_or(&empty)
        .to_str()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .expect("Password not set");

    let user_id = match token::verify_jwt_token(token) {
        Ok(id) => id,
        Err(why) => {
            eprintln!("Error {}", why);
            return Err((StatusCode::UNAUTHORIZED, "Token was not verified").into_response());
        }
    };

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let dir_name = user_id.to_string();

        let path = std::path::Path::new(&dir_name);
        if !path.exists() {
            std::fs::create_dir(&dir_name).unwrap();
        }
        let full_path = dir_name.to_string() + "/" + STORAGE_FILENAME;
        let mut file = match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(full_path)
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Server error. Couldnt upload",
                )
            }) {
            Ok(file) => file,
            Err(why) => return Err(why.into_response()),
        };

        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?
        {
            file.write(&chunk).unwrap();
        }
        file.flush().unwrap();
    }
    let full_path = user_id.to_string() + "/" + PASSWORD_FILENAME;
    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(full_path)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Server error. Couldnt upload",
            )
        }) {
        Ok(file) => file,
        Err(why) => return Err(why.into_response()),
    };
    file.write(password_of_storage.as_bytes()).unwrap();
    file.flush().unwrap();

    Ok((StatusCode::OK, "Successfully uploaded storage").into_response())
}

pub async fn download(headers: HeaderMap) -> Result<impl IntoResponse, (StatusCode, String)> {
    let empty = HeaderValue::from_str("").unwrap();

    let token = headers
        .get("Authorization")
        .unwrap_or(&empty)
        .to_str()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .expect("Token is not set");

    let user_id = match token::verify_jwt_token(token) {
        Ok(id) => id,
        Err(why) => {
            eprintln!("Error {}", why);
            return Err((StatusCode::UNAUTHORIZED, "Token was not verified".into()));
        }
    };

    let mut password_str = String::new();

    {
        let path = user_id.to_string() + "/" + PASSWORD_FILENAME;
        let mut file = match tokio::fs::File::open(path).await {
            Ok(file) => file,
            Err(why) => return Err((StatusCode::NOT_FOUND, why.to_string())),
        };
        file.read_to_string(&mut password_str).await.unwrap();
        file.flush().await.unwrap();
    }

    let path = user_id.to_string() + "/" + STORAGE_FILENAME;
    let file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(why) => return Err((StatusCode::NOT_FOUND, why.to_string())),
    };

    let stream = tokio_util::io::ReaderStream::with_capacity(file, 4096);
    let stream_body = axum::body::Body::from_stream(stream);

    Ok(axum::response::Response::builder()
        .header(
            CONTENT_TYPE,
            HeaderValue::from_str("application/vnd.sqlite3").unwrap(),
        )
        .header(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(
                format!("form-data; name=\"user\"; filename=\"pmanager.pm\"").as_str(),
            )
            .unwrap(),
        )
        .header("Password", HeaderValue::from_str(&password_str).unwrap())
        .body(stream_body)
        .unwrap())
}
