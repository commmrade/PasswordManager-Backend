use sqlx::MySqlPool;

pub async fn create_token(
    pool: &MySqlPool,
    user_id: u32,
    refresh_token: &str,
) -> anyhow::Result<u32> {
    let row = sqlx::query!(
        "INSERT INTO refresh_tokens (user_id, token) VALUES (?, ?)",
        user_id,
        refresh_token
    )
    .execute(pool)
    .await?;
    Ok(row.last_insert_id() as u32)
}

pub async fn token_exists(pool: &MySqlPool, token: &str) -> anyhow::Result<()> {
    sqlx::query("SELECT * FROM refresh_tokens WHERE token = ?")
        .bind(token)
        .fetch_one(pool)
        .await?;
    Ok(())
}

pub async fn delete_token(pool: &MySqlPool, token: &str) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM refresh_tokens WHERE token = ?", token)
        .execute(pool)
        .await?;
    Ok(())
}
