use anyhow::{Context, Result};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::PathBuf;

/// Returns the path to the progit data directory (~/.progit/).
pub fn data_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let dir = home.join(".progit");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("Could not create data directory: {}", dir.display()))?;
    Ok(dir)
}

/// Opens (or creates) the SQLite database and runs all pending migrations.
pub async fn init_db() -> Result<SqlitePool> {
    let db_path = data_dir()?.join("database.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .with_context(|| format!("Failed to open database at {}", db_path.display()))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run database migrations")?;

    Ok(pool)
}
