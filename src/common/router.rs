use std::time::Duration;

use axum::{
    error_handling::HandleErrorLayer, extract::DefaultBodyLimit, http::StatusCode, BoxError, Router,
};
use sqlx::{MySql, MySqlPool, Pool};
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};
use tower_http::{catch_panic::CatchPanicLayer, cors::CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    common::{
        error::{internal_server_error_handler, ErrorResponse, ErrorTypes},
        swagger::ApiDoc,
    },
    handlers, AppState,
};

fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", axum::routing::post(handlers::auth::register))
        .route("/login", axum::routing::post(handlers::auth::login))
        .route(
            "/token",
            axum::routing::get(handlers::auth::update_jwt_token),
        )
        .route("/validate", axum::routing::get(handlers::auth::validate))
        .route("/logout", axum::routing::post(handlers::auth::logout))
}

fn storage_routes() -> Router<AppState> {
    Router::new()
        .route("/download", axum::routing::get(handlers::storage::download))
        .route("/upload", axum::routing::post(handlers::storage::upload))
}

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .merge(auth_routes())
        .merge(storage_routes())
        .layer(DefaultBodyLimit::max(1 * 1024 * 1024 * 1024 * 2))
        .layer(CorsLayer::permissive()) // Для того чтоб CORS мозг не ебал
        .layer(CatchPanicLayer::custom(internal_server_error_handler))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    // So compiler wont complain about some Infallable Trait shit
                    eprintln!("{}", err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(ErrorResponse::new(
                            ErrorTypes::InternalError,
                            "Internal error occured",
                        )),
                    )
                }))
                .layer(BufferLayer::new(1024)) // Means it can process 1024 messages before backpressure is applied TODO: Adjust
                .layer(RateLimitLayer::new(5, Duration::from_secs(1))), // Rate limti does not impl Clone, so we need to use BufferLayer TODO: Adjust
        )
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
