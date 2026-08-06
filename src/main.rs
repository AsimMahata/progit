use anyhow::Result;
use clap::Parser;
use chrono::Local;

mod cli;
mod commands;
mod db;
mod display;
mod models;

use cli::{Cli, Commands, TaskAction};
use commands::{cf, edit, lc, note, task, view};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let pool = db::init_db().await?;

    match cli.command {
        // ── Codeforces ────────────────────────────────────────────────────────
        Commands::Cf { help_me, rating, difficulty, rest, date, time } => {
            // `progit cf help` → no rating, first rest token is "help"
            let first = rest.first().map(|s| s.as_str()).unwrap_or("");
            if help_me || (rating.is_none() && first == "help") {
                cf::print_cf_help();
            } else if rating.is_none() && first == "list" {
                // `progit cf list [--all]`
                let show_all = rest.iter().any(|s| s == "--all");
                let entries = cf::list_cf(&pool, show_all).await?;
                display::print_activity_list(&entries, "codeforces");
            } else {
                cf::handle_cf(&pool, rating, difficulty, rest, date, time).await?;
            }
        }

        // ── LeetCode ──────────────────────────────────────────────────────────
        Commands::Lc { difficulty, topic, notes, date, time } => {
            match difficulty.as_deref() {
                Some("help") => lc::print_lc_help(),
                Some("list") => {
                    // `progit lc list [--all]`  — topic holds "--all" if present
                    let show_all = topic.as_deref() == Some("--all");
                    let entries = lc::list_lc(&pool, show_all).await?;
                    display::print_activity_list(&entries, "leetcode");
                }
                _ => lc::handle_lc(&pool, difficulty, topic, notes, date, time).await?,
            }
        }

        // ── Task subcommands ──────────────────────────────────────────────────
        Commands::Task { action } => match action {
            TaskAction::Add => {
                task::handle_task_add(&pool).await?;
            }
            TaskAction::Edit { id } => {
                task::handle_task_edit(&pool, id).await?;
            }
            TaskAction::Status { id, status } => {
                task::handle_task_status(&pool, id, status).await?;
            }
            TaskAction::List { filter } => {
                let tasks = task::handle_tasks_list(&pool, filter.as_deref()).await?;
                display::print_task_list(&tasks);
            }
        },

        // ── Tasks shorthand ───────────────────────────────────────────────────
        Commands::Tasks { todo, doing, done, cancelled } => {
            let filter = if todo      { Some("Todo") }
                         else if doing     { Some("Doing") }
                         else if done      { Some("Done") }
                         else if cancelled { Some("Cancelled") }
                         else             { None };
            let tasks = task::handle_tasks_list(&pool, filter).await?;
            display::print_task_list(&tasks);
        }

        // ── Note ──────────────────────────────────────────────────────────────
        Commands::Note { text, date, time } => {
            note::handle_note(&pool, text, date, time).await?;
        }

        // ── View: today ───────────────────────────────────────────────────────
        Commands::Today => {
            let today = Local::now().format("%Y-%m-%d").to_string();
            let v = view::build_day_view(&pool, &today).await?;
            display::print_day(&v);
        }

        // ── View: yesterday ───────────────────────────────────────────────────
        Commands::Yesterday => {
            use chrono::Duration;
            let yesterday = (Local::now() - Duration::days(1))
                .format("%Y-%m-%d")
                .to_string();
            let v = view::build_day_view(&pool, &yesterday).await?;
            display::print_day(&v);
        }

        // ── View: specific date ───────────────────────────────────────────────
        Commands::Date { date } => {
            let v = view::build_day_view(&pool, &date).await?;
            display::print_day(&v);
        }

        // ── View: last N days ─────────────────────────────────────────────────
        Commands::Last { days } => {
            let views = view::build_last_n_days(&pool, days).await?;
            display::print_days(&views);
        }

        // ── cf list / lc list (hidden top-level aliases) ─────────────────────
        Commands::CfList { all } => {
            let entries = cf::list_cf(&pool, all).await?;
            display::print_activity_list(&entries, "codeforces");
        }

        Commands::LcList { all } => {
            let entries = lc::list_lc(&pool, all).await?;
            display::print_activity_list(&entries, "leetcode");
        }

        // ── Edit activity by ID ───────────────────────────────────────────────
        Commands::Edit { id, date, time, rating, difficulty, notes, tags, topic, lc_difficulty } => {
            edit::handle_edit_activity(
                &pool, id, date, time, rating, difficulty, notes, tags, topic, lc_difficulty,
            ).await?;
        }

        // ── Edit note by ID ───────────────────────────────────────────────────
        Commands::EditNote { id, text, date, time } => {
            edit::handle_edit_note(&pool, id, text, date, time).await?;
        }

        // ── Uninstall ─────────────────────────────────────────────────────────
        Commands::Uninstall => {
            edit::handle_uninstall()?;
        }
    }

    Ok(())
}
