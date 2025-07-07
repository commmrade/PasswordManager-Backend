use sqlx::MySqlPool;

use crate::database;

pub async fn create_user(
    state: &MySqlPool,
    username: &str,
    email: &str,
    password_hash: &str,
) -> anyhow::Result<u32> {
    let id = database::users::create_user(state, username, email, password_hash).await?;
    Ok(id)
}

pub async fn id_by_email(state: &MySqlPool, email: &str) -> anyhow::Result<u32> {
    let id = database::users::id_by_email(state, email).await?;
    Ok(id)
}

pub async fn get_password_hash(state: &MySqlPool, id: u32) -> anyhow::Result<String> {
    let pwd_hash = database::users::get_password_hash(state, id).await?;
    Ok(pwd_hash)
}

pub async fn set_nonce(state: &MySqlPool, nonce: &[u8], id: u32) -> anyhow::Result<()> {
    database::users::set_nonce(state, nonce, id).await?;
    Ok(())
}

pub async fn get_nonce(state: &MySqlPool, id: u32) -> anyhow::Result<Vec<u8>> {
    let nonce = database::users::nonce(state, id).await?;
    Ok(nonce)
}
