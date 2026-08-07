use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

// ─── Backup directory ──────

/// Returns ~/tools/progit/, creating it if absent.
/// This directory is NEVER deleted by `progit uninstall`.
pub fn backup_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let dir = home.join("tools").join("progit");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("Could not create backup directory: {}", dir.display()))?;
    Ok(dir)
}

// ─── Append helpers 

fn append_line(filename: &str, line: &str) -> Result<()> {
    let path = backup_dir()?.join(filename);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("Could not open backup file: {}", path.display()))?;
    writeln!(file, "{}", line)
        .with_context(|| format!("Could not write to backup file: {}", path.display()))?;
    Ok(())
}

/// Append a Codeforces entry to codeforces.txt
pub fn append_cf(
    rating: i64,
    difficulty: i64,
    tags: &[String],
    notes: Option<&str>,
    date: &str,
    time: &str,
) -> Result<()> {
    let stars = difficulty_stars(difficulty);
    let tags_str = if tags.is_empty() {
        "–".to_string()
    } else {
        tags.join(", ")
    };
    let notes_str = notes.unwrap_or("–");
    let line = format!(
        "[{} {}] CF | Rating: {} | Diff: {} | Tags: {} | Notes: {}",
        date, time, rating, stars, tags_str, notes_str
    );
    append_line("codeforces.txt", &line)
}

/// Append a LeetCode entry to leetcode.txt
pub fn append_lc(
    lc_difficulty: &str,
    topic: Option<&str>,
    notes: Option<&str>,
    date: &str,
    time: &str,
) -> Result<()> {
    let topic_str = topic.unwrap_or("–");
    let notes_str = notes.unwrap_or("–");
    let line = format!(
        "[{} {}] LC | {} | Topic: {} | Notes: {}",
        date, time, lc_difficulty, topic_str, notes_str
    );
    append_line("leetcode.txt", &line)
}

/// Append a note entry to notes.txt
pub fn append_note(text: &str, date: &str, time: &str) -> Result<()> {
    let line = format!("[{} {}] NOTE | {}", date, time, text);
    append_line("notes.txt", &line)
}

// ─── Helpers ───────

fn difficulty_stars(d: i64) -> String {
    let d = d.clamp(1, 5) as usize;
    format!("{}{}", "★".repeat(d), "☆".repeat(5 - d))
}
