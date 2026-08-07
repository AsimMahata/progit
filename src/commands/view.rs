use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::models::{Activity, DayView, Note, Task};

// ─── Query helpers ─

async fn activities_for_date(pool: &SqlitePool, date: &str) -> Result<Vec<Activity>> {
    sqlx::query_as::<_, Activity>(
        "SELECT * FROM activities WHERE date = ? ORDER BY time ASC",
    )
    .bind(date)
    .fetch_all(pool)
    .await
    .context("Failed to query activities")
}

async fn tasks_active(pool: &SqlitePool) -> Result<Vec<Task>> {
    sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks
         WHERE LOWER(COALESCE(status,'')) NOT IN ('done','cancelled')
         ORDER BY priority DESC, created_at DESC",
    )
    .fetch_all(pool)
    .await
    .context("Failed to query tasks")
}

async fn notes_for_date(pool: &SqlitePool, date: &str) -> Result<Vec<Note>> {
    sqlx::query_as::<_, Note>(
        "SELECT * FROM notes WHERE date = ? ORDER BY time ASC",
    )
    .bind(date)
    .fetch_all(pool)
    .await
    .context("Failed to query notes")
}

// ─── DayView builder ──────

pub async fn build_day_view(pool: &SqlitePool, date: &str) -> Result<DayView> {
    let activities = activities_for_date(pool, date).await?;
    let tasks      = tasks_active(pool).await?;
    let notes      = notes_for_date(pool, date).await?;
    Ok(DayView { date: date.to_string(), activities, tasks, notes })
}

// ─── Multi-day view 

pub async fn build_last_n_days(pool: &SqlitePool, days: u32) -> Result<Vec<DayView>> {
    use chrono::{Duration, Local};
    let today = Local::now().date_naive();
    let mut views = Vec::new();

    for i in 0..days {
        let d = today - Duration::days(i as i64);
        let date_str = d.format("%Y-%m-%d").to_string();
        let view = build_day_view(pool, &date_str).await?;
        views.push(view);
    }
    views.reverse();
    Ok(views)
}
