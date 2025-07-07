use std::{fs::OpenOptions, io::Write};

use axum::{
    extract::{Multipart, State},
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::{IntoResponse, Response},
};
use minio::s3::{
    builders::ObjectContent,
    types::{PartInfo, S3Api},
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use tokio_util::io::ReaderStream;

const STORAGE_FILENAME: &str = "pmanager.pm";

use crate::{
    common::error::{error_response, AppError, ErrorTypes},
    crypt::token::AuthHeader,
    AppState,
};

#[derive(Serialize, Deserialize)]
pub struct UploadReq {
    password: String,
}

pub async fn upload(
    State(s3_client): State<minio::s3::Client>,
    auth_header: AuthHeader,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let user_id = auth_header.claims.id;

    let filename = user_id.to_string() + "/pmanager.pm";

    while let Some(mut field) = multipart.next_field().await? {
        let multipart_upload = s3_client
            .create_multipart_upload("user-storages", &filename)
            .send()
            .await?;
        let mut parts: Vec<minio::s3::types::PartInfo> = Vec::new();

        while let Some(chunk) = field.chunk().await? {
            let response: minio::s3::response::UploadPartResponse = s3_client
                .upload_part(
                    &multipart_upload.bucket,
                    &multipart_upload.object,
                    &multipart_upload.upload_id,
                    (parts.len() + 1) as u16, // Indexing of parts start at 1
                    chunk.clone().into(),
                )
                .send()
                .await?;

            let partinfo = PartInfo {
                number: (parts.len() + 1) as u16,
                etag: response.etag,
                size: chunk.len() as u64,
            };
            parts.push(partinfo);
        }
        let _: minio::s3::response::CompleteMultipartUploadResponse = s3_client
            .complete_multipart_upload(
                "user-storages",
                &filename,
                multipart_upload.upload_id,
                parts,
            )
            .send()
            .await?;
    }

    Ok((StatusCode::OK).into_response())
}

pub async fn download(
    State(s3_client): State<minio::s3::Client>,
    auth_header: AuthHeader,
) -> Result<Response, AppError> {
    let user_id = auth_header.claims.id;

    let filename = user_id.to_string() + "/pmanager.pm";
    let response = match s3_client
        .get_object("user-storages", &filename)
        .send()
        .await
    {
        Ok(response) => response,
        Err(why) => {
            tracing::error!("Such file does not exist");
            return Ok(crate::error_response!(
                StatusCode::NOT_FOUND,
                ErrorTypes::FileNotExists,
                "Such file does not exist: {}",
                why
            ));
        }
    };

    let (stream, _size) = response.content.to_stream().await?;
    let body = axum::body::Body::from_stream(stream);

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
        .body(body)?;
    Ok(response)
}
