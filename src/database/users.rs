use sqlx::MySqlPool;
pub async fn create_user(
    pool: &MySqlPool,
    username: &str,
    email: &str,
    password_hash: &str,
) -> anyhow::Result<u32> {
    let row = sqlx::query!(
        "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)",
        username,
        email,
        password_hash
    )
    .execute(pool)
    .await?;
    Ok(row.last_insert_id() as u32)
}

pub async fn id_by_email(pool: &MySqlPool, email: &str) -> anyhow::Result<u32> {
    let row = sqlx::query!("SELECT id FROM users WHERE email = ?", email)
        .fetch_one(pool)
        .await?;
    Ok(row.id as u32)
}

pub async fn get_password_hash(pool: &MySqlPool, id: u32) -> anyhow::Result<String> {
    let row = sqlx::query!("SELECT password_hash FROM users WHERE id = ?", id)
        .fetch_one(pool)
        .await?;
    Ok(row.password_hash)
}
