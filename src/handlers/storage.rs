use std::{fs::OpenOptions, io::Write};

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

const STORAGE_FILENAME: &str = "pmanager.pm";

use crate::{common::error::AppError, crypt::token::AuthHeader};

#[derive(Serialize, Deserialize)]
pub struct UploadReq {
    password: String,
}

pub async fn upload(
    State(_): State<MySqlPool>,
    auth_header: AuthHeader,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
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

    Ok((StatusCode::OK).into_response())
}

pub async fn download(
    State(_): State<MySqlPool>,
    auth_header: AuthHeader,
) -> Result<Response, AppError> {
    let user_id = auth_header.claims.id;

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
        .body(stream_body)?;
    Ok(response)
}
