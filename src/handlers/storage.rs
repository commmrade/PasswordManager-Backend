use std::{fs::OpenOptions, io::Write};

use anyhow::anyhow;
use axum::{
    extract::{Multipart, State},
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const STORAGE_FILENAME: &str = "pmanager.pm";
const PASSWORD_FILENAME: &str = "password.txt";

use crate::{
    common::error::AppError,
    controllers,
    crypt::{encryption, token::AuthHeader},
};

#[derive(Serialize, Deserialize)]
pub struct UploadReq {
    password: String,
}

#[axum::debug_handler]
pub async fn upload(
    State(pool): State<MySqlPool>,
    auth_header: AuthHeader,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let password_of_storage = headers
        .get("Password")
        .ok_or(anyhow!("fuck"))?
        .to_str()
        .expect("Password not set");
    let user_id = auth_header.claims.id;

    while let Some(mut field) = multipart.next_field().await? {
        let dir_name = user_id.to_string();

        let path = std::path::Path::new(&dir_name);
        if !path.exists() {
            std::fs::create_dir(&dir_name)?;
        }
        let full_path = dir_name.to_string() + "/" + STORAGE_FILENAME;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(full_path)?;

        while let Some(chunk) = field.chunk().await? {
            file.write(&chunk)?;
        }
        file.flush()?;
    }

    let full_path = user_id.to_string() + "/" + PASSWORD_FILENAME;
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(full_path)?;

    // TODO: Make it safer
    let encrypted = encryption::aes_encrypt_text(password_of_storage)?;
    controllers::users::set_nonce(&pool, &encrypted.1, user_id).await?;

    file.write(encrypted.0.as_slice())?;
    file.flush()?;

    Ok((StatusCode::OK).into_response())
}

pub async fn download(
    State(pool): State<MySqlPool>,
    auth_header: AuthHeader,
) -> Result<Response, AppError> {
    let user_id = auth_header.claims.id;

    let path = user_id.to_string() + "/" + PASSWORD_FILENAME;
    let mut file = tokio::fs::File::open(path).await?;

    let mut raw_password_str = Vec::new();
    file.read_to_end(&mut raw_password_str).await?;

    let nonce = controllers::users::get_nonce(&pool, user_id).await?; // Must exist because user at this point is there
    let password_str = encryption::aes_decrypt_text(&raw_password_str, &nonce)?;

    file.flush().await?;
    drop(file);

    let path = user_id.to_string() + "/" + STORAGE_FILENAME;
    let file = tokio::fs::File::open(path).await?;

    let stream = tokio_util::io::ReaderStream::with_capacity(file, 4096);
    let stream_body = axum::body::Body::from_stream(stream);

    let response = axum::response::Response::builder()
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
        .body(stream_body)?;
    Ok(response)
}
