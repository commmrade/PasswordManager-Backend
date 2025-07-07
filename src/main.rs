use std::time::Duration;

use axum::extract::FromRef;
use minio::s3::{
    builders::ListBuckets, creds::StaticProvider, response::ListBucketsResponse, types::S3Api,
    ClientBuilder,
};
use sqlx::{mysql::MySqlPoolOptions, MySql, MySqlPool, Pool};

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

#[derive(Clone)]
struct AppState {
    pool: MySqlPool,
    s3_client: minio::s3::Client,
}

impl FromRef<AppState> for Pool<MySql> {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<AppState> for minio::s3::Client {
    fn from_ref(state: &AppState) -> Self {
        state.s3_client.clone()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let connection_str = std::env::var("DATABASE_URL").expect("DATABASE_URL NOT SET");
    let mysql_pool = get_pool(&connection_str).await;

    let static_provider = StaticProvider::new("klewy", "dvfu1312", None);
    let client = ClientBuilder::new("http://localhost:9000".parse().unwrap())
        .provider(Some(Box::new(static_provider)))
        .build()
        .unwrap();

    let state = AppState {
        pool: mysql_pool,
        s3_client: client,
    };

    let app = common::router::get_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    println!("Hello, world!");
}
