use sqlx::MySqlPool;
use sqlx::Row;

pub async fn create_user(
    pool: &MySqlPool,
    username: &str,
    email: &str,
    password_hash: &str,
) -> anyhow::Result<u32> {
    sqlx::query("INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)")
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .execute(pool)
        .await?;

    let user_id = sqlx::query("SELECT LAST_INSERT_ID()")
        .fetch_one(pool)
        .await?
        .try_get::<u64, _>(0)?;

    Ok(user_id as u32)
}

pub async fn id_by_email(pool: &MySqlPool, email: &str) -> anyhow::Result<u32> {
    let row = sqlx::query("SELECT id FROM users WHERE email = ?")
        .bind(email)
        .fetch_one(pool)
        .await?;
    let id: i32 = row.try_get(0)?;
    Ok(id as u32)
}

pub async fn get_password_hash(pool: &MySqlPool, id: u32) -> anyhow::Result<String> {
    let row = sqlx::query("SELECT password_hash FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;
    let hash: String = row.try_get(0)?;
    Ok(hash)
}
