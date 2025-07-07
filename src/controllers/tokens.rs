use sqlx::MySqlPool;

use crate::database;

pub async fn create_token(
    state: &MySqlPool,
    user_id: u32,
    refresh_token: &str,
) -> anyhow::Result<u32> {
    let id = database::tokens::create_token(state, user_id, refresh_token).await?;
    Ok(id)
}

pub async fn token_exists(state: &MySqlPool, token: &str) -> anyhow::Result<()> {
    database::tokens::token_exists(state, token).await
}

pub async fn delete_token(state: &MySqlPool, token: &str) -> anyhow::Result<()> {
    database::tokens::delete_token(state, token).await
}
