use anyhow::{Context, Result};
use chrono::Local;
use owo_colors::OwoColorize;
use sqlx::SqlitePool;

use crate::models::Activity;

// ─── Contextual help ─────────────────────────────────────────────────────────

pub fn print_cf_help() {
    println!();
    println!("{}", "  ⚡ progit cf — Log a Codeforces problem".cyan().bold());
    println!();
    println!("  {}", "Usage:".bold());
    println!("    progit cf <rating> <difficulty> [tags...] [\"notes\"]");
    println!();
    println!("  {}", "Fields:".bold());
    println!("    rating      Problem rating (e.g. 1700, 1900, 2100)");
    println!("    difficulty  1=Easy  2=Medium  3=Hard  4=Very Hard  5=Insane");
    println!("    tags        Any space-separated tokens (E1, WA3, BinarySearch, CHT, ...)");
    println!("    notes       Quoted string at the end (optional)");
    println!();
    println!("  {}", "Options:".bold());
    println!("    -d, --date <YYYY-MM-DD>   Override date (default: today)");
    println!("    -t, --time <HH:MM>        Override time (default: now)");
    println!();
    println!("  {}", "Examples:".bold());
    println!("    progit cf 1700 1 E1");
    println!("    progit cf 1900 3 E2 T5");
    println!("    progit cf 2100 5 CHT \"Needed editorial\"");
    println!("    progit cf 1800 3 WA3 BinarySearch \"Off-by-one in binary search\"");
    println!();
}

// ─── Command handler ─────────────────────────────────────────────────────────

pub async fn handle_cf(
    pool: &SqlitePool,
    rating: Option<i64>,
    difficulty: Option<i64>,
    rest: Vec<String>,
    date: Option<String>,
    time: Option<String>,
) -> Result<()> {
    let rating = match rating {
        Some(r) => r,
        None => {
            print_cf_help();
            println!(
                "  {} rating is required. Example: {}",
                "Missing:".yellow().bold(),
                "progit cf 1700 2 E1".cyan()
            );
            println!();
            return Ok(());
        }
    };

    let difficulty = match difficulty {
        Some(d) => d,
        None => {
            print_cf_help();
            println!(
                "  {} difficulty (1–5) is required. Example: {}",
                "Missing:".yellow().bold(),
                format!("progit cf {} 2 E1", rating).cyan()
            );
            println!();
            return Ok(());
        }
    };

    if !(1..=5).contains(&difficulty) {
        eprintln!("Difficulty must be between 1 and 5. Got: {}", difficulty);
        return Ok(());
    }

    // Parse rest: extract --date/--time flags if present (trailing_var_arg swallows them),
    // bare tokens → tags, quoted/space-containing strings → notes.
    let mut tags: Vec<String> = Vec::new();
    let mut notes: Option<String> = None;
    let mut rest_date: Option<String> = None;
    let mut rest_time: Option<String> = None;

    let mut iter = rest.iter().peekable();
    while let Some(token) = iter.next() {
        if token == "--date" || token == "-d" {
            rest_date = iter.next().cloned();
        } else if token == "--time" || token == "-t" {
            rest_time = iter.next().cloned();
        } else if let Some(val) = token.strip_prefix("--date=") {
            rest_date = Some(val.to_string());
        } else if let Some(val) = token.strip_prefix("--time=") {
            rest_time = Some(val.to_string());
        } else if token.contains(' ') {
            notes = Some(token.clone());
        } else {
            tags.push(token.clone());
        }
    }

    let tags_json: Option<String> = if tags.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&tags)?)
    };

    let now = Local::now();
    // Explicit --date/--time flags take priority, then any found in rest, then now
    let date = date.or(rest_date).unwrap_or_else(|| now.format("%Y-%m-%d").to_string());
    let time = time.or(rest_time).unwrap_or_else(|| now.format("%H:%M").to_string());

    sqlx::query(
        "INSERT INTO activities (platform, date, time, difficulty, rating, tags, notes)
         VALUES ('codeforces', ?, ?, ?, ?, ?, ?)",
    )
    .bind(&date)
    .bind(&time)
    .bind(difficulty)
    .bind(rating)
    .bind(&tags_json)
    .bind(&notes)
    .execute(pool)
    .await
    .context("Failed to insert Codeforces activity")?;

    println!();
    println!(
        "  {} {} {} {}",
        "✓".green().bold(),
        "Codeforces".cyan().bold(),
        format!("Rating {} | Difficulty {}", rating, difficulty).bold(),
        format!("| {}", date).dimmed()
    );
    if !tags.is_empty() {
        println!("    Tags:  {}", tags.join(", ").dimmed());
    }
    if let Some(n) = &notes {
        println!("    Notes: {}", n.dimmed());
    }
    println!();

    Ok(())
}

/// List recent Codeforces entries.
pub async fn list_cf(pool: &SqlitePool, show_all: bool) -> Result<Vec<Activity>> {
    let sql = if show_all {
        "SELECT * FROM activities WHERE platform = 'codeforces' ORDER BY date DESC, time DESC"
    } else {
        "SELECT * FROM activities WHERE platform = 'codeforces' ORDER BY date DESC, time DESC LIMIT 20"
    };
    let rows = sqlx::query_as::<_, Activity>(sql)
        .fetch_all(pool)
        .await
        .context("Failed to list Codeforces activities")?;
    Ok(rows)
}
