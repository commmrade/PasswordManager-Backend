use std::time::Duration;

use sqlx::{mysql::MySqlPoolOptions, MySqlPool};

mod common;
mod controllers;
mod crypt;
mod database;
mod handlers;

async fn get_pool(connection_str: &str) -> MySqlPool {
    MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(10))
        .connect(connection_str)
        .await
        .expect("Cant connect")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let connection_str = std::env::var("DATABASE_URL").expect("DATABASE_URL NOT SET");
    let mysql_pool = get_pool(&connection_str).await;
    let app = common::router::get_router(mysql_pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    println!("Hello, world!");
}
