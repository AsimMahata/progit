use anyhow::{Context, Result};
use chrono::Local;
use owo_colors::OwoColorize;
use sqlx::SqlitePool;

pub async fn handle_note(
    pool: &SqlitePool,
    text: String,
    date: Option<String>,
    time: Option<String>,
) -> Result<()> {
    let now = Local::now();
    let date = date.unwrap_or_else(|| now.format("%Y-%m-%d").to_string());
    let time = time.unwrap_or_else(|| now.format("%H:%M").to_string());

    sqlx::query("INSERT INTO notes (date, time, text) VALUES (?, ?, ?)")
        .bind(&date)
        .bind(&time)
        .bind(&text)
        .execute(pool)
        .await
        .context("Failed to insert note")?;

    println!();
    println!(
        "  {} {} {}",
        "✓".green().bold(),
        "Note logged".magenta().bold(),
        format!("| {}", date).dimmed()
    );
    println!("    {}", text.dimmed());
    println!();

    Ok(())
}
