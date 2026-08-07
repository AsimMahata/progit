use anyhow::{Context, Result};
use chrono::Local;
use owo_colors::OwoColorize;
use sqlx::SqlitePool;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use crate::backup::backup_dir;

// ─── Directory helpers ────────────────────────────────────────────────────────

fn current_dir() -> Result<PathBuf> {
    let d = backup_dir()?.join("current");
    fs::create_dir_all(&d).context("Could not create current/ directory")?;
    Ok(d)
}

fn checkpoints_dir() -> Result<PathBuf> {
    let d = backup_dir()?.join("backups");
    fs::create_dir_all(&d).context("Could not create backups/ directory")?;
    Ok(d)
}

fn checkpoint_dir(name: &str) -> Result<PathBuf> {
    let d = checkpoints_dir()?.join(name);
    fs::create_dir_all(&d)
        .with_context(|| format!("Could not create backup directory: {}", d.display()))?;
    Ok(d)
}

// ─── Dump DB → directory of .txt files ───────────────────────────────────────

async fn dump_db_to_dir(pool: &SqlitePool, dir: &Path, timestamp: &str) -> Result<()> {
    dump_activities(pool, dir, timestamp).await?;
    dump_notes(pool, dir, timestamp).await?;
    dump_tasks(pool, dir, timestamp).await?;
    Ok(())
}

async fn dump_activities(pool: &SqlitePool, dir: &Path, timestamp: &str) -> Result<()> {
    let rows: Vec<(i64, String, String, Option<String>, Option<i64>, Option<i64>,
                   Option<String>, Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT id, platform, date, time, difficulty, rating,
                    lc_difficulty, topic, tags, notes
             FROM activities ORDER BY id ASC"
        )
        .fetch_all(pool)
        .await
        .context("Failed to read activities")?;

    // ── activities.txt ────────────────────────────────────────────────────────
    let mut out = String::new();
    out.push_str(&format!("# Progit Activities Backup — {}\n", timestamp));
    out.push_str("# Fields: id|platform|date|time|difficulty|rating|lc_difficulty|topic|tags|notes\n");
    for (id, platform, date, time, diff, rating, lc_diff, topic, tags, notes) in &rows {
        out.push_str(&format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
            id,
            platform,
            date,
            time.as_deref().unwrap_or(""),
            diff.map(|d| d.to_string()).unwrap_or_default(),
            rating.map(|r| r.to_string()).unwrap_or_default(),
            lc_diff.as_deref().unwrap_or(""),
            topic.as_deref().unwrap_or(""),
            // Encode tags JSON so pipes inside don't break parsing
            tags.as_deref().unwrap_or("").replace('|', "\\|"),
            notes.as_deref().unwrap_or("").replace('|', "\\|"),
        ));
    }
    fs::write(dir.join("activities.txt"), &out).context("Failed to write activities.txt")?;
    Ok(())
}

async fn dump_notes(pool: &SqlitePool, dir: &Path, timestamp: &str) -> Result<()> {
    let rows: Vec<(i64, String, Option<String>, String)> =
        sqlx::query_as("SELECT id, date, time, text FROM notes ORDER BY id ASC")
            .fetch_all(pool)
            .await
            .context("Failed to read notes")?;

    let mut out = String::new();
    out.push_str(&format!("# Progit Notes Backup — {}\n", timestamp));
    out.push_str("# Fields: id|date|time|text\n");
    for (id, date, time, text) in &rows {
        out.push_str(&format!(
            "{}|{}|{}|{}\n",
            id,
            date,
            time.as_deref().unwrap_or(""),
            text.replace('|', "\\|"),
        ));
    }
    fs::write(dir.join("notes.txt"), &out).context("Failed to write notes.txt")?;
    Ok(())
}

async fn dump_tasks(pool: &SqlitePool, dir: &Path, timestamp: &str) -> Result<()> {
    let rows: Vec<(i64, String, Option<String>, Option<String>, i64, Option<String>)> =
        sqlx::query_as(
            "SELECT id, title, description, status, priority, deadline
             FROM tasks ORDER BY id ASC"
        )
        .fetch_all(pool)
        .await
        .context("Failed to read tasks")?;

    let mut out = String::new();
    out.push_str(&format!("# Progit Tasks Backup — {}\n", timestamp));
    out.push_str("# Fields: id|title|description|status|priority|deadline\n");
    for (id, title, desc, status, priority, deadline) in &rows {
        out.push_str(&format!(
            "{}|{}|{}|{}|{}|{}\n",
            id,
            title.replace('|', "\\|"),
            desc.as_deref().unwrap_or("").replace('|', "\\|"),
            status.as_deref().unwrap_or(""),
            priority,
            deadline.as_deref().unwrap_or(""),
        ));
    }
    fs::write(dir.join("tasks.txt"), &out).context("Failed to write tasks.txt")?;
    Ok(())
}

// ─── Restore DB from a directory of .txt files ───────────────────────────────

async fn restore_db_from_dir(pool: &SqlitePool, dir: &Path) -> Result<()> {
    // Wipe existing data
    sqlx::query("DELETE FROM activities").execute(pool).await?;
    sqlx::query("DELETE FROM notes").execute(pool).await?;
    sqlx::query("DELETE FROM tasks").execute(pool).await?;
    // Reset autoincrement sequences
    sqlx::query("DELETE FROM sqlite_sequence WHERE name IN ('activities','notes','tasks')")
        .execute(pool).await.ok(); // ok() — sqlite_sequence may not exist if no rows ever inserted

    restore_activities(pool, dir).await?;
    restore_notes(pool, dir).await?;
    restore_tasks(pool, dir).await?;
    Ok(())
}

fn parse_field(s: &str) -> String {
    s.replace("\\|", "|")
}

async fn restore_activities(pool: &SqlitePool, dir: &Path) -> Result<()> {
    let path = dir.join("activities.txt");
    if !path.exists() { return Ok(()); }

    let file   = fs::File::open(&path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') || line.trim().is_empty() { continue; }
        let parts: Vec<&str> = line.splitn(10, '|').collect();
        if parts.len() < 10 { continue; }

        let id:         i64            = parts[0].parse().unwrap_or(0);
        let platform:   &str           = parts[1];
        let date:       &str           = parts[2];
        let time:       Option<String> = non_empty(parts[3]);
        let difficulty: Option<i64>    = parts[4].parse().ok();
        let rating:     Option<i64>    = parts[5].parse().ok();
        let lc_diff:    Option<String> = non_empty(parts[6]);
        let topic:      Option<String> = non_empty(&parse_field(parts[7]));
        let tags:       Option<String> = non_empty(&parse_field(parts[8]));
        let notes:      Option<String> = non_empty(&parse_field(parts[9].trim_end_matches('\n')));

        sqlx::query(
            "INSERT OR REPLACE INTO activities
             (id, platform, date, time, difficulty, rating, lc_difficulty, topic, tags, notes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(id).bind(platform).bind(date).bind(time)
        .bind(difficulty).bind(rating).bind(lc_diff)
        .bind(topic).bind(tags).bind(notes)
        .execute(pool).await?;
    }
    Ok(())
}

async fn restore_notes(pool: &SqlitePool, dir: &Path) -> Result<()> {
    let path = dir.join("notes.txt");
    if !path.exists() { return Ok(()); }

    let file   = fs::File::open(&path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') || line.trim().is_empty() { continue; }
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() < 4 { continue; }

        let id:   i64            = parts[0].parse().unwrap_or(0);
        let date: &str           = parts[1];
        let time: Option<String> = non_empty(parts[2]);
        let text: String         = parse_field(parts[3].trim_end_matches('\n'));

        sqlx::query("INSERT OR REPLACE INTO notes (id, date, time, text) VALUES (?, ?, ?, ?)")
            .bind(id).bind(date).bind(time).bind(text)
            .execute(pool).await?;
    }
    Ok(())
}

async fn restore_tasks(pool: &SqlitePool, dir: &Path) -> Result<()> {
    let path = dir.join("tasks.txt");
    if !path.exists() { return Ok(()); }

    let file   = fs::File::open(&path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') || line.trim().is_empty() { continue; }
        let parts: Vec<&str> = line.splitn(6, '|').collect();
        if parts.len() < 6 { continue; }

        let id:       i64            = parts[0].parse().unwrap_or(0);
        let title:    String         = parse_field(parts[1]);
        let desc:     Option<String> = non_empty(&parse_field(parts[2]));
        let status:   Option<String> = non_empty(parts[3]);
        let priority: i64            = parts[4].parse().unwrap_or(1);
        let deadline: Option<String> = non_empty(parts[5].trim_end_matches('\n'));

        sqlx::query(
            "INSERT OR REPLACE INTO tasks (id, title, description, status, priority, deadline)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(id).bind(title).bind(desc).bind(status).bind(priority).bind(deadline)
        .execute(pool).await?;
    }
    Ok(())
}

fn non_empty(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() { None } else { Some(t.to_string()) }
}

// ─── Public command handlers ──────────────────────────────────────────────────

/// `progit backup [name]`
pub async fn handle_backup(pool: &SqlitePool, name: Option<String>) -> Result<()> {
    let now       = Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M").to_string();
    let date_str  = now.format("%Y-%m-%d").to_string();

    let checkpoint_name = name.unwrap_or_else(|| date_str.clone());

    println!();
    println!(
        "  {} {}  {}",
        "📦".cyan(),
        "Creating backup".bold(),
        format!("\"{}\"", checkpoint_name).white()
    );
    println!();

    // 1. Named checkpoint backup
    let chk_dir = checkpoint_dir(&checkpoint_name)?;
    dump_db_to_dir(pool, &chk_dir, &timestamp).await?;
    println!(
        "  {} Checkpoint → {}",
        "✓".green().bold(),
        chk_dir.display().to_string().cyan()
    );

    // 2. Overwrite current/ with latest state
    let cur_dir = current_dir()?;
    dump_db_to_dir(pool, &cur_dir, &timestamp).await?;
    println!(
        "  {} Current   → {}",
        "✓".green().bold(),
        cur_dir.display().to_string().dimmed()
    );

    // Show what was backed up
    let cf_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM activities WHERE platform='codeforces'"
    ).fetch_one(pool).await.unwrap_or((0,));
    let lc_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM activities WHERE platform='leetcode'"
    ).fetch_one(pool).await.unwrap_or((0,));
    let note_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notes"
    ).fetch_one(pool).await.unwrap_or((0,));
    let task_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tasks"
    ).fetch_one(pool).await.unwrap_or((0,));

    println!();
    println!(
        "    {} CF   {}  {} LC   {}  {} Notes {}  {} Tasks {}",
        "⚡".cyan(),  cf_count.0.to_string().cyan(),
        "📘".yellow(), lc_count.0.to_string().yellow(),
        "📝".magenta(), note_count.0.to_string().magenta(),
        "✅".green(), task_count.0.to_string().green(),
    );
    println!();

    Ok(())
}

/// `progit restore <name>`
pub async fn handle_restore(pool: &SqlitePool, name: String) -> Result<()> {
    // Find the checkpoint
    let chk_dir = checkpoints_dir()?.join(&name);
    if !chk_dir.exists() {
        // Try to list available backups
        let bk_root = checkpoints_dir()?;
        let mut available: Vec<String> = Vec::new();
        if let Ok(entries) = fs::read_dir(&bk_root) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    available.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        available.sort();

        println!();
        println!(
            "  {} No backup named \"{}\" found.",
            "✗".red().bold(),
            name.yellow()
        );
        if available.is_empty() {
            println!("    No backups exist yet. Run {} first.", "progit backup".cyan());
        } else {
            println!("  {} Available backups:", "·".bright_black());
            for b in &available {
                println!("      {}", b.cyan());
            }
        }
        println!();
        return Ok(());
    }

    println!();
    println!(
        "  {} Preparing to restore from \"{}\"",
        "⚠".yellow().bold(),
        name.yellow().bold()
    );
    println!("  {} {} {}",
        "·".bright_black(),
        "This will REPLACE your current database.".red().bold(),
        "Make sure you have a backup.".dimmed()
    );
    println!();

    // Ask if they have a current backup
    let cur_dir = current_dir()?;
    let has_current = cur_dir.join("activities.txt").exists();

    if has_current {
        print!(
            "  {} A 'current' snapshot exists. Proceed with restore? {}  ",
            "📦".cyan(),
            "[y/n]".bold()
        );
        io::stdout().flush().ok();
        let answer = read_line_trimmed()?;
        if !answer.eq_ignore_ascii_case("y") {
            println!();
            println!("  {} Restore cancelled.", "·".dimmed());
            println!();
            return Ok(());
        }
    } else {
        // No current snapshot — offer to make one first
        print!(
            "  {} No current backup found. Create one before restoring? {}  ",
            "⚠".yellow(),
            "[y/n]".bold()
        );
        io::stdout().flush().ok();
        let answer = read_line_trimmed()?;

        if answer.eq_ignore_ascii_case("y") {
            print!(
                "  {} Backup name {} ",
                "·".bright_black(),
                format!("[default: {}]: ", Local::now().format("%Y-%m-%d")).dimmed()
            );
            io::stdout().flush().ok();
            let raw = read_line_trimmed()?;
            let bk_name = if raw.is_empty() {
                Local::now().format("%Y-%m-%d").to_string()
            } else {
                raw
            };
            handle_backup(pool, Some(bk_name)).await?;
        } else {
            // No backup — final confirmation
            print!(
                "  {} Restore WITHOUT a backup? This cannot be undone. {}  ",
                "🔥".red(),
                "[y/n]".red().bold()
            );
            io::stdout().flush().ok();
            let answer = read_line_trimmed()?;
            if !answer.eq_ignore_ascii_case("y") {
                println!();
                println!("  {} Restore cancelled.", "·".dimmed());
                println!();
                return Ok(());
            }
        }
    }

    // ── Perform restore ───────────────────────────────────────────────────────
    println!();
    println!("  {} Restoring from \"{}\" ...", "⟳".cyan(), name.cyan().bold());

    restore_db_from_dir(pool, &chk_dir).await
        .context("Restore failed — your database may be partially overwritten; check the backup files")?;

    // Count what was restored
    let cf_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM activities WHERE platform='codeforces'"
    ).fetch_one(pool).await.unwrap_or((0,));
    let lc_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM activities WHERE platform='leetcode'"
    ).fetch_one(pool).await.unwrap_or((0,));
    let note_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notes"
    ).fetch_one(pool).await.unwrap_or((0,));
    let task_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tasks"
    ).fetch_one(pool).await.unwrap_or((0,));

    println!(
        "  {} Restore complete!",
        "✓".green().bold()
    );
    println!();
    println!(
        "    {} CF   {}  {} LC   {}  {} Notes {}  {} Tasks {}",
        "⚡".cyan(),   cf_count.0.to_string().cyan(),
        "📘".yellow(), lc_count.0.to_string().yellow(),
        "📝".magenta(), note_count.0.to_string().magenta(),
        "✅".green(),  task_count.0.to_string().green(),
    );
    println!();

    Ok(())
}

/// `progit backups` — list all available checkpoints
pub fn handle_list_backups() -> Result<()> {
    let root = backup_dir()?;
    let bk_root = root.join("backups");
    let cur_dir  = root.join("current");

    println!();
    println!("  {} {}", "📦", "BACKUPS".white().bold());
    println!("  {}", "────────────────────────────────────────".bright_black());

    // Current snapshot
    if cur_dir.join("activities.txt").exists() {
        let meta = fs::metadata(cur_dir.join("activities.txt")).ok();
        let modified = meta
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let dt: chrono::DateTime<Local> = t.into();
                dt.format("%Y-%m-%d %H:%M").to_string()
            })
            .unwrap_or_else(|| "–".to_string());
        println!(
            "  {} {}  {}",
            "●".cyan().bold(),
            "current".cyan().bold(),
            format!("(last updated: {})", modified).bright_black()
        );
    } else {
        println!("  {} {}", "○".bright_black(), "current  (none)".dimmed());
    }

    // Named checkpoints
    if bk_root.exists() {
        let mut entries: Vec<_> = fs::read_dir(&bk_root)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().is_dir())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        if !entries.is_empty() {
            println!();
            for entry in &entries {
                let n = entry.file_name().to_string_lossy().to_string();
                let meta = fs::metadata(entry.path().join("activities.txt")).ok();
                let modified = meta
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        let dt: chrono::DateTime<Local> = t.into();
                        dt.format("%Y-%m-%d %H:%M").to_string()
                    })
                    .unwrap_or_else(|| "–".to_string());
                println!(
                    "  {} {}  {}",
                    "▸".yellow(),
                    n.yellow().bold(),
                    format!("({})", modified).bright_black()
                );
            }
        } else {
            println!();
            println!("  {}", "No named checkpoints yet.".dimmed());
        }
    }

    println!();
    println!("  {}", "Run `progit backup [name]` to create a checkpoint.".dimmed());
    println!("  {}", "Run `progit restore <name>` to restore.".dimmed());
    println!();
    Ok(())
}

// ─── stdin helper ─────────────────────────────────────────────────────────────

fn read_line_trimmed() -> Result<String> {
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .context("Failed to read input")?;
    Ok(buf.trim().to_string())
}
