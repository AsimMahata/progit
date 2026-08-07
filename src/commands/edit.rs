use anyhow::{bail, Context, Result};
use owo_colors::OwoColorize;
use sqlx::SqlitePool;

use crate::models::{Activity, Note};

// ─── Edit activity ─

pub async fn handle_edit_activity(
    pool: &SqlitePool,
    id: i64,
    date: Option<String>,
    time: Option<String>,
    rating: Option<i64>,
    difficulty: Option<i64>,
    notes: Option<String>,
    tags: Option<String>,
    topic: Option<String>,
    lc_difficulty: Option<String>,
) -> Result<()> {
    // Fetch existing row
    let existing = sqlx::query_as::<_, Activity>("SELECT * FROM activities WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .context("Failed to fetch activity")?;

    let existing = match existing {
        Some(a) => a,
        None    => bail!("No activity found with id {}. Use `progit cf list` or `progit lc list` to see IDs.", id),
    };

    // Merge: use provided value, else keep existing
    let new_date       = date.unwrap_or(existing.date.clone());
    let new_time       = time.or(existing.time.clone());
    let new_rating     = rating.or(existing.rating);
    let new_difficulty = difficulty.or(existing.difficulty);
    let new_notes      = notes.or(existing.notes.clone());
    let new_topic      = topic.or(existing.topic.clone());
    let new_lc_diff    = lc_difficulty.or(existing.lc_difficulty.clone());

    // Tags: comma-separated string → JSON array, or keep existing
    let new_tags = if let Some(t) = tags {
        let tag_vec: Vec<String> = t
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if tag_vec.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&tag_vec)?)
        }
    } else {
        existing.tags.clone()
    };

    sqlx::query(
        "UPDATE activities
         SET date=?, time=?, rating=?, difficulty=?, lc_difficulty=?, topic=?, tags=?, notes=?
         WHERE id=?",
    )
    .bind(&new_date)
    .bind(&new_time)
    .bind(new_rating)
    .bind(new_difficulty)
    .bind(&new_lc_diff)
    .bind(&new_topic)
    .bind(&new_tags)
    .bind(&new_notes)
    .bind(id)
    .execute(pool)
    .await
    .context("Failed to update activity")?;

    // Re-fetch to show what it looks like now
    let updated = sqlx::query_as::<_, Activity>("SELECT * FROM activities WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    println!();
    println!(
        "  {} Activity #{} updated  [{}]",
        "✓".green().bold(),
        id,
        updated.platform.cyan()
    );
    println!("    Date:       {}", updated.date);
    if let Some(t) = &updated.time       { println!("    Time:       {}", t); }
    if let Some(r) = updated.rating      { println!("    Rating:     {}", r); }
    if updated.difficulty.is_some() { println!("    Difficulty: {}", updated.difficulty_stars()); }
    if let Some(ld) = &updated.lc_diff_display() { println!("    LC Diff:    {}", ld); }
    if let Some(tp) = &updated.topic     { println!("    Topic:      {}", tp); }
    let tags = updated.parse_tags();
    if !tags.is_empty()                  { println!("    Tags:       {}", tags.join(", ")); }
    if let Some(n) = &updated.notes      { println!("    Notes:      {}", n); }
    println!();

    Ok(())
}

// ─── Edit note ─────

pub async fn handle_edit_note(
    pool: &SqlitePool,
    id: i64,
    text: Option<String>,
    date: Option<String>,
    time: Option<String>,
) -> Result<()> {
    let existing = sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .context("Failed to fetch note")?;

    let existing = match existing {
        Some(n) => n,
        None    => bail!("No note found with id {}.", id),
    };

    let new_text = text.unwrap_or(existing.text.clone());
    let new_date = date.unwrap_or(existing.date.clone());
    let new_time = time.or(existing.time.clone());

    sqlx::query("UPDATE notes SET text=?, date=?, time=? WHERE id=?")
        .bind(&new_text)
        .bind(&new_date)
        .bind(&new_time)
        .bind(id)
        .execute(pool)
        .await
        .context("Failed to update note")?;

    println!();
    println!(
        "  {} Note #{} updated",
        "✓".green().bold(),
        id
    );
    println!("    {} — {}", new_date.dimmed(), new_text);
    println!();

    Ok(())
}

// ─── Uninstall ───

pub fn handle_uninstall() -> Result<()> {
    println!();
    println!("  {} {}", "⚠".yellow().bold(), "Uninstalling progit".bold());
    println!();

    // Remove data directory (SQLite database)
    let data_dir = crate::db::data_dir()?;
    if data_dir.exists() {
        std::fs::remove_dir_all(&data_dir)
            .with_context(|| format!("Could not remove {}", data_dir.display()))?;
        println!("  {} Removed data directory: {}", "✓".green(), data_dir.display());
    } else {
        println!("  {} Data directory not found (already removed?)", "·".dimmed());
    }

    // Remove binary from ~/.cargo/bin/
    if let Some(home) = dirs::home_dir() {
        let bin_name = if cfg!(windows) { "progit.exe" } else { "progit" };
        let bin_path = home.join(".cargo").join("bin").join(bin_name);
        if bin_path.exists() {
            std::fs::remove_file(&bin_path)
                .with_context(|| format!("Could not remove binary at {}", bin_path.display()))?;
            println!("  {} Removed binary: {}", "✓".green(), bin_path.display());
        } else {
            println!("  {} Binary not found at {} (installed differently?)", "·".dimmed(), bin_path.display());
        }
    }

    // ── Backup preserved ───────
    if let Some(home) = dirs::home_dir() {
        let backup_path = home.join("tools").join("progit");
        if backup_path.exists() {
            println!();
            println!(
                "  {} {}",
                "📦".cyan(),
                "Backup data preserved — your logs are safe:".bold()
            );
            println!(
                "     {}",
                backup_path.display().to_string().cyan()
            );
            println!("     (codeforces.txt  leetcode.txt  notes.txt)");
            println!("     This folder was NOT deleted. Reinstall progit anytime to keep tracking.");
        }
    }

    println!();
    println!("  {} Progit uninstalled. Goodbye!", "✓".green().bold());
    println!();

    Ok(())
}
