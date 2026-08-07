use anyhow::{bail, Context, Result};
use owo_colors::OwoColorize;
use sqlx::SqlitePool;
use std::io::{self, Write};

use crate::models::{parse_priority, Task};

// ─── Prompt helpers 

fn prompt(label: &str) -> Result<String> {
    print!("{} ", label.bold());
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

fn prompt_optional(label: &str, hint: &str) -> Result<Option<String>> {
    print!("{} {} ", label.bold(), hint.dimmed());
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let s = buf.trim().to_string();
    Ok(if s.is_empty() { None } else { Some(s) })
}

// ─── Add task ──────

pub async fn handle_task_add(pool: &SqlitePool) -> Result<()> {
    println!();
    println!("{}", "  ✅ Add Task".green().bold());
    println!();

    let title = loop {
        let t = prompt("  Title:")?;
        if !t.is_empty() { break t; }
        println!("  {} Title cannot be empty.", "!".yellow());
    };

    let description = prompt_optional("  Description:", "(optional, press Enter to skip)")?;

    println!();
    println!("  {}", "Status (optional):".bold());
    println!("    Suggested: Todo · Doing · Done · Cancelled  (or type any custom value)");
    let status = prompt_optional("  Status:", "(press Enter to leave blank)")?;

    println!();
    println!("  {}", "Priority:".bold());
    println!("    1  Low  (default)");
    println!("    2  Medium");
    println!("    3  High");
    let priority: i64 = loop {
        let p = prompt("  Priority [1]:")?;
        if p.is_empty() { break 1; }
        match parse_priority(&p) {
            Some(v) => break v,
            None    => println!("  {} Enter 1, 2, or 3 (or low/medium/high).", "!".yellow()),
        }
    };

    let deadline = prompt_optional("  Deadline:", "(YYYY-MM-DD, optional)")?;

    sqlx::query(
        "INSERT INTO tasks (title, description, status, priority, deadline)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&title)
    .bind(&description)
    .bind(&status)
    .bind(priority)
    .bind(&deadline)
    .execute(pool)
    .await
    .context("Failed to insert task")?;

    println!();
    println!("  {} Task '{}' added.", "✓".green().bold(), title.bold());
    println!();

    Ok(())
}

// ─── Edit task ─────

pub async fn handle_task_edit(pool: &SqlitePool, id: i64) -> Result<()> {
    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .context("Failed to fetch task")?;

    let task = match task {
        Some(t) => t,
        None    => bail!("No task found with id {}", id),
    };

    println!();
    println!(
        "  {} Editing Task #{}  (press Enter to keep current value)",
        "✏".yellow().bold(),
        id
    );
    println!();

    // Title
    print!("  Title [{}]: ", task.title.bold());
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let title = {
        let t = buf.trim();
        if t.is_empty() { task.title.clone() } else { t.to_string() }
    };

    // Description
    let desc_hint = task.description.as_deref().unwrap_or("");
    print!("  Description [{}]: ", desc_hint.bold());
    io::stdout().flush()?;
    buf.clear();
    io::stdin().read_line(&mut buf)?;
    let desc_input = buf.trim().to_string();
    let description: Option<String> = if desc_input.is_empty() {
        task.description.clone()
    } else if desc_input == "-" {
        None
    } else {
        Some(desc_input)
    };

    // Status
    let status_hint = task.status.as_deref().unwrap_or("");
    print!("  Status [{}]: ", status_hint.bold());
    io::stdout().flush()?;
    buf.clear();
    io::stdin().read_line(&mut buf)?;
    let status_input = buf.trim().to_string();
    let status: Option<String> = if status_input.is_empty() {
        task.status.clone()
    } else if status_input == "-" {
        None
    } else {
        Some(status_input)
    };

    // Priority
    println!("  Priority: 1=Low  2=Medium  3=High");
    print!("  Priority [{}]: ", task.priority_label().bold());
    io::stdout().flush()?;
    buf.clear();
    io::stdin().read_line(&mut buf)?;
    let priority_input = buf.trim().to_string();
    let priority = if priority_input.is_empty() {
        task.priority
    } else {
        parse_priority(&priority_input).unwrap_or_else(|| {
            println!("  {} Invalid priority, keeping current.", "!".yellow());
            task.priority
        })
    };

    // Deadline
    let dl_hint = task.deadline.as_deref().unwrap_or("none");
    print!("  Deadline [{}] (- to clear): ", dl_hint.bold());
    io::stdout().flush()?;
    buf.clear();
    io::stdin().read_line(&mut buf)?;
    let dl_input = buf.trim().to_string();
    let deadline: Option<String> = if dl_input.is_empty() {
        task.deadline.clone()
    } else if dl_input == "-" {
        None
    } else {
        Some(dl_input)
    };

    sqlx::query(
        "UPDATE tasks SET title=?, description=?, status=?, priority=?, deadline=? WHERE id=?",
    )
    .bind(&title)
    .bind(&description)
    .bind(&status)
    .bind(priority)
    .bind(&deadline)
    .bind(id)
    .execute(pool)
    .await
    .context("Failed to update task")?;

    println!();
    println!("  {} Task #{} updated.", "✓".green().bold(), id);
    println!();

    Ok(())
}

// ─── Set status ────

pub async fn handle_task_status(pool: &SqlitePool, id: i64, status: String) -> Result<()> {
    let result = sqlx::query("UPDATE tasks SET status = ? WHERE id = ?")
        .bind(&status)
        .bind(id)
        .execute(pool)
        .await
        .context("Failed to update task status")?;

    if result.rows_affected() == 0 {
        bail!("No task found with id {}", id);
    }

    println!();
    println!("  {} Task #{} → {}", "✓".green().bold(), id, status.bold());
    println!();

    Ok(())
}

// ─── List tasks ────

pub async fn handle_tasks_list(pool: &SqlitePool, filter: Option<&str>) -> Result<Vec<Task>> {
    let tasks = if let Some(f) = filter {
        sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE LOWER(status) = LOWER(?)
             ORDER BY priority DESC, created_at DESC",
        )
        .bind(f)
        .fetch_all(pool)
        .await
        .context("Failed to query tasks")?
    } else {
        sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks ORDER BY priority DESC, created_at DESC",
        )
        .fetch_all(pool)
        .await
        .context("Failed to query tasks")?
    };
    Ok(tasks)
}
