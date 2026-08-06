use anyhow::{Context, Result};
use chrono::Local;
use owo_colors::OwoColorize;
use sqlx::SqlitePool;

use crate::models::Activity;

// ─── Contextual help ─────────────────────────────────────────────────────────

pub fn print_lc_help() {
    println!();
    println!("{}", "  📘 progit lc — Log a LeetCode problem".yellow().bold());
    println!();
    println!("  {}", "Usage:".bold());
    println!("    progit lc <difficulty> [topic] [\"notes\"]");
    println!();
    println!("  {}", "Fields:".bold());
    println!("    difficulty  Easy | Medium | Hard");
    println!("    topic       Optional topic tag (e.g. Graph, DP, SegmentTree)");
    println!("    notes       Quoted string at the end (optional)");
    println!();
    println!("  {}", "Options:".bold());
    println!("    -d, --date <YYYY-MM-DD>   Override date (default: today)");
    println!("    -t, --time <HH:MM>        Override time (default: now)");
    println!();
    println!("  {}", "Examples:".bold());
    println!("    progit lc Easy");
    println!("    progit lc Hard Graph");
    println!("    progit lc Medium DP");
    println!("    progit lc Hard SegmentTree \"Needed hints\"");
    println!();
}

// ─── Difficulty mapping ───────────────────────────────────────────────────────

fn lc_difficulty_to_int(d: &str) -> Option<i64> {
    match d.to_lowercase().as_str() {
        "easy"   => Some(2),
        "medium" => Some(3),
        "hard"   => Some(4),
        _        => None,
    }
}

fn normalize_difficulty(d: &str) -> &'static str {
    match d.to_lowercase().as_str() {
        "easy"   => "Easy",
        "medium" => "Medium",
        "hard"   => "Hard",
        _        => "Unknown",
    }
}

// ─── Command handler ─────────────────────────────────────────────────────────

pub async fn handle_lc(
    pool: &SqlitePool,
    difficulty: Option<String>,
    topic: Option<String>,
    notes: Option<String>,
    date: Option<String>,
    time: Option<String>,
) -> Result<()> {
    let difficulty_str = match difficulty {
        Some(d) => d,
        None => {
            print_lc_help();
            println!(
                "  {} difficulty is required (Easy | Medium | Hard). Example: {}",
                "Missing:".yellow().bold(),
                "progit lc Medium DP".yellow()
            );
            println!();
            return Ok(());
        }
    };

    let difficulty_int = match lc_difficulty_to_int(&difficulty_str) {
        Some(d) => d,
        None => {
            print_lc_help();
            println!(
                "  {} '{}' is not a valid difficulty. Use: {}",
                "Invalid:".red().bold(),
                difficulty_str,
                "Easy | Medium | Hard".yellow()
            );
            println!();
            return Ok(());
        }
    };

    let lc_difficulty = normalize_difficulty(&difficulty_str);

    let now = Local::now();
    let date = date.unwrap_or_else(|| now.format("%Y-%m-%d").to_string());
    let time = time.unwrap_or_else(|| now.format("%H:%M").to_string());

    sqlx::query(
        "INSERT INTO activities (platform, date, time, difficulty, lc_difficulty, topic, notes)
         VALUES ('leetcode', ?, ?, ?, ?, ?, ?)",
    )
    .bind(&date)
    .bind(&time)
    .bind(difficulty_int)
    .bind(lc_difficulty)
    .bind(&topic)
    .bind(&notes)
    .execute(pool)
    .await
    .context("Failed to insert LeetCode activity")?;

    let difficulty_colored = match lc_difficulty {
        "Easy"   => "Easy".green().to_string(),
        "Medium" => "Medium".yellow().to_string(),
        "Hard"   => "Hard".red().to_string(),
        _        => lc_difficulty.to_string(),
    };

    println!();
    println!(
        "  {} {} {} {}",
        "✓".green().bold(),
        "LeetCode".yellow().bold(),
        difficulty_colored.bold(),
        format!("| {}", date).dimmed()
    );
    if let Some(t) = &topic {
        println!("    Topic: {}", t.dimmed());
    }
    if let Some(n) = &notes {
        println!("    Notes: {}", n.dimmed());
    }
    println!();

    Ok(())
}

/// List recent LeetCode entries.
pub async fn list_lc(pool: &SqlitePool, show_all: bool) -> Result<Vec<Activity>> {
    let sql = if show_all {
        "SELECT * FROM activities WHERE platform = 'leetcode' ORDER BY date DESC, time DESC"
    } else {
        "SELECT * FROM activities WHERE platform = 'leetcode' ORDER BY date DESC, time DESC LIMIT 20"
    };
    let rows = sqlx::query_as::<_, Activity>(sql)
        .fetch_all(pool)
        .await
        .context("Failed to list LeetCode activities")?;
    Ok(rows)
}
