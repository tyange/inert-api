use sqlx::{Pool, Sqlite};

pub async fn init_db(db: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            user_id     TEXT PRIMARY KEY,
            email       TEXT UNIQUE NOT NULL,
            username    TEXT UNIQUE NOT NULL,
            password    TEXT NOT NULL,
            display_name TEXT,
            avatar_url  TEXT,
            bio         TEXT,
            created_at  TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS stills (
            still_id    TEXT PRIMARY KEY,
            slug        TEXT UNIQUE NOT NULL,
            user_id     TEXT NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
            caption     TEXT,
            published_at TEXT NOT NULL DEFAULT (datetime('now')),
            created_at  TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS still_images (
            image_id    TEXT PRIMARY KEY,
            still_id    TEXT NOT NULL REFERENCES stills(still_id) ON DELETE CASCADE,
            image_url   TEXT NOT NULL,
            image_key   TEXT NOT NULL,
            width       INTEGER,
            height      INTEGER,
            position    INTEGER NOT NULL DEFAULT 0,
            created_at  TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(db)
    .await?;

    // google_sub 컬럼 마이그레이션 (없으면 추가)
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN google_sub TEXT")
        .execute(db)
        .await;

    Ok(())
}
