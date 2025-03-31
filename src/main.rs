use std::time::Duration;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};

pub mod controllers;
pub mod crypt;
pub mod handlers;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let connect_str = "mysql://klewy:root@localhost:3306/pm";

    let mysql_pool = MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(10))
        .connect(connect_str)
        .await
        .expect("Cant connect");

    let app = Router::new()
        .route("/register", post(handlers::auth::register))
        .route("/login", post(handlers::auth::login))
        .route("/token", get(handlers::auth::token))
        .route("/download", get(handlers::storage::download))
        .route("/upload", post(handlers::storage::upload))
        .route("/validate", get(handlers::auth::validate))
        .layer(DefaultBodyLimit::max(1 * 1024 * 1024 * 1024 * 2))
        .with_state(mysql_pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    println!("Hello, world!");
}
