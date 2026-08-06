use chrono::NaiveDate;
use comfy_table::{presets::UTF8_BORDERS_ONLY, Attribute, Cell, Color, Table};
use owo_colors::OwoColorize;

use crate::models::{Activity, DayView, Task};

// ─── Shared layout constants ─────────────────────────────────────────────────

const DIVIDER: &str = "────────────────────────────────────────";
const HEADER:  &str = "════════════════════════════════════════";

// ─── Date formatting ─────────────────────────────────────────────────────────

fn format_date_heading(date: &str) -> String {
    if let Ok(d) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        d.format("%d %B %Y").to_string()
    } else {
        date.to_string()
    }
}

// ─── Day view printer ─────────────────────────────────────────────────────────

pub fn print_day(view: &DayView) {
    let has_anything = !view.activities.is_empty()
        || !view.tasks.is_empty()
        || !view.notes.is_empty();

    // Header
    println!();
    println!("  {}", HEADER.bright_black());
    println!(
        "  {}  {}",
        "📅",
        format_date_heading(&view.date).white().bold()
    );
    println!("  {}", HEADER.bright_black());

    if !has_anything {
        println!();
        println!("  {}", "No activity logged for this day.".dimmed());
        println!();
        return;
    }

    // ── Codeforces ───────────────────────────────────────────────────────────
    let cf_entries: Vec<&Activity> = view
        .activities
        .iter()
        .filter(|a| a.platform == "codeforces")
        .collect();

    if !cf_entries.is_empty() {
        println!();
        println!("  {} {}", "⚡", "CODEFORCES".cyan().bold());
        println!("  {}", DIVIDER.bright_black());

        let mut table = Table::new();
        table.load_preset(UTF8_BORDERS_ONLY);
        table.set_header(vec![
            Cell::new("ID").add_attribute(Attribute::Bold),
            Cell::new("Date").add_attribute(Attribute::Bold),
            Cell::new("Time").add_attribute(Attribute::Bold),
            Cell::new("Rating").add_attribute(Attribute::Bold),
            Cell::new("Difficulty").add_attribute(Attribute::Bold),
            Cell::new("Tags").add_attribute(Attribute::Bold),
            Cell::new("Notes").add_attribute(Attribute::Bold),
        ]);

        for entry in cf_entries {
            let time    = entry.time.as_deref().unwrap_or("–");
            let rating  = entry
                .rating
                .map(|r| r.to_string())
                .unwrap_or_else(|| "–".to_string());
            let stars   = entry.difficulty_stars();
            let tags    = entry.parse_tags().join(", ");
            let notes   = entry.notes.as_deref().unwrap_or("");

            table.add_row(vec![
                Cell::new(entry.id).fg(Color::DarkGrey),
                Cell::new(&entry.date).fg(Color::DarkGrey),
                Cell::new(time),
                Cell::new(rating).fg(Color::Cyan),
                Cell::new(stars).fg(Color::Yellow),
                Cell::new(tags).fg(Color::DarkGrey),
                Cell::new(notes).fg(Color::DarkGrey),
            ]);
        }

        // Indent each line of the table
        for line in table.to_string().lines() {
            println!("    {}", line);
        }
    }

    // ── LeetCode ─────────────────────────────────────────────────────────────
    let lc_entries: Vec<&Activity> = view
        .activities
        .iter()
        .filter(|a| a.platform == "leetcode")
        .collect();

    if !lc_entries.is_empty() {
        println!();
        println!("  {} {}", "📘", "LEETCODE".yellow().bold());
        println!("  {}", DIVIDER.bright_black());

        let mut table = Table::new();
        table.load_preset(UTF8_BORDERS_ONLY);
        table.set_header(vec![
            Cell::new("ID").add_attribute(Attribute::Bold),
            Cell::new("Date").add_attribute(Attribute::Bold),
            Cell::new("Time").add_attribute(Attribute::Bold),
            Cell::new("Difficulty").add_attribute(Attribute::Bold),
            Cell::new("Topic").add_attribute(Attribute::Bold),
            Cell::new("Notes").add_attribute(Attribute::Bold),
        ]);

        for entry in lc_entries {
            let time  = entry.time.as_deref().unwrap_or("–");
            let diff  = entry.lc_difficulty.as_deref().unwrap_or("–");
            let topic = entry.topic.as_deref().unwrap_or("");
            let notes = entry.notes.as_deref().unwrap_or("");

            let diff_cell = match diff {
                "Easy"   => Cell::new(diff).fg(Color::Green),
                "Medium" => Cell::new(diff).fg(Color::Yellow),
                "Hard"   => Cell::new(diff).fg(Color::Red),
                _        => Cell::new(diff),
            };

            table.add_row(vec![
                Cell::new(entry.id).fg(Color::DarkGrey),
                Cell::new(&entry.date).fg(Color::DarkGrey),
                Cell::new(time),
                diff_cell,
                Cell::new(topic).fg(Color::DarkGrey),
                Cell::new(notes).fg(Color::DarkGrey),
            ]);
        }

        for line in table.to_string().lines() {
            println!("    {}", line);
        }
    }

    // ── Tasks ─────────────────────────────────────────────────────────────────
    if !view.tasks.is_empty() {
        println!();
        println!("  {} {}", "✅", "TASKS".green().bold());
        println!("  {}", DIVIDER.bright_black());

        let mut table = Table::new();
        table.load_preset(UTF8_BORDERS_ONLY);
        table.set_header(vec![
            Cell::new("ID").add_attribute(Attribute::Bold),
            Cell::new("Title").add_attribute(Attribute::Bold),
            Cell::new("Status").add_attribute(Attribute::Bold),
            Cell::new("Priority").add_attribute(Attribute::Bold),
            Cell::new("Deadline").add_attribute(Attribute::Bold),
        ]);

        for task in &view.tasks {
            let status_cell = match task.status.as_deref() {
                Some("Todo")      => Cell::new("Todo").fg(Color::DarkGrey),
                Some("Doing")     => Cell::new("Doing").fg(Color::Yellow),
                Some("Done")      => Cell::new("Done").fg(Color::Green),
                Some("Cancelled") => Cell::new("Cancelled").fg(Color::Red),
                Some(s)           => Cell::new(s).fg(Color::Cyan),
                None              => Cell::new("–").fg(Color::DarkGrey),
            };

            let priority_cell = match task.priority {
                3 => Cell::new("High").fg(Color::Red),
                2 => Cell::new("Medium").fg(Color::Yellow),
                _ => Cell::new("Low").fg(Color::DarkGrey),
            };

            table.add_row(vec![
                Cell::new(task.id),
                Cell::new(&task.title),
                status_cell,
                priority_cell,
                Cell::new(task.deadline.as_deref().unwrap_or("–")).fg(Color::DarkGrey),
            ]);
        }

        for line in table.to_string().lines() {
            println!("    {}", line);
        }
    }

    // ── Notes ─────────────────────────────────────────────────────────────────
    if !view.notes.is_empty() {
        println!();
        println!("  {} {}", "📝", "NOTES".magenta().bold());
        println!("  {}", DIVIDER.bright_black());
        for note in &view.notes {
            let time_part = note
                .time
                .as_deref()
                .map(|t| format!(" [{}]", t))
                .unwrap_or_default();
            println!(
                "    {} #{}{} • {}",
                "▸".bright_black(),
                note.id.to_string().bright_black(),
                time_part.bright_black(),
                note.text
            );
        }
    }

    println!();
    println!("  {}", HEADER.bright_black());
    println!();
}

// ─── Multi-day view ───────────────────────────────────────────────────────────

pub fn print_days(views: &[DayView]) {
    for view in views {
        print_day(view);
    }
}

// ─── Task list view ───────────────────────────────────────────────────────────

pub fn print_task_list(tasks: &[Task]) {
    if tasks.is_empty() {
        println!();
        println!("  {}", "No tasks found.".dimmed());
        println!();
        return;
    }

    println!();
    println!("  {} {}", "✅", "TASKS".green().bold());
    println!("  {}", DIVIDER.bright_black());

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_header(vec![
        Cell::new("ID").add_attribute(Attribute::Bold),
        Cell::new("Title").add_attribute(Attribute::Bold),
        Cell::new("Description").add_attribute(Attribute::Bold),
        Cell::new("Status").add_attribute(Attribute::Bold),
        Cell::new("Priority").add_attribute(Attribute::Bold),
        Cell::new("Deadline").add_attribute(Attribute::Bold),
    ]);

    for task in tasks {
        let status_cell = match task.status.as_deref() {
            Some("Todo")      => Cell::new("Todo").fg(Color::DarkGrey),
            Some("Doing")     => Cell::new("Doing").fg(Color::Yellow),
            Some("Done")      => Cell::new("Done").fg(Color::Green),
            Some("Cancelled") => Cell::new("Cancelled").fg(Color::Red),
            Some(s)           => Cell::new(s).fg(Color::Cyan),
            None              => Cell::new("–").fg(Color::DarkGrey),
        };

        let priority_cell = match task.priority {
            3 => Cell::new("High").fg(Color::Red),
            2 => Cell::new("Medium").fg(Color::Yellow),
            _ => Cell::new("Low").fg(Color::DarkGrey),
        };

        table.add_row(vec![
            Cell::new(task.id),
            Cell::new(&task.title),
            Cell::new(task.description.as_deref().unwrap_or("")).fg(Color::DarkGrey),
            status_cell,
            priority_cell,
            Cell::new(task.deadline.as_deref().unwrap_or("–")).fg(Color::DarkGrey),
        ]);
    }

    for line in table.to_string().lines() {
        println!("    {}", line);
    }
    println!();
}

// ─── Activity list view (cf list / lc list) ──────────────────────────────────

pub fn print_activity_list(activities: &[Activity], platform: &str) {
    if activities.is_empty() {
        println!();
        println!("  {}", format!("No {} entries found.", platform).dimmed());
        println!();
        return;
    }

    let (icon, label, _color) = match platform {
        "codeforces" => ("⚡", "CODEFORCES", Color::Cyan),
        "leetcode"   => ("📘", "LEETCODE",   Color::Yellow),
        _            => ("·",  platform,     Color::White),
    };

    println!();
    println!("  {} {}", icon, label.bold().to_string().cyan());
    println!("  {}", DIVIDER.bright_black());

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);

    if platform == "codeforces" {
        table.set_header(vec![
            Cell::new("Date").add_attribute(Attribute::Bold),
            Cell::new("Time").add_attribute(Attribute::Bold),
            Cell::new("Rating").add_attribute(Attribute::Bold),
            Cell::new("Difficulty").add_attribute(Attribute::Bold),
            Cell::new("Tags").add_attribute(Attribute::Bold),
            Cell::new("Notes").add_attribute(Attribute::Bold),
        ]);
        for a in activities {
            table.add_row(vec![
                Cell::new(&a.date),
                Cell::new(a.time.as_deref().unwrap_or("–")),
                Cell::new(a.rating.map(|r| r.to_string()).unwrap_or_else(|| "–".into())).fg(Color::Cyan),
                Cell::new(a.difficulty_stars()).fg(Color::Yellow),
                Cell::new(a.parse_tags().join(", ")).fg(Color::DarkGrey),
                Cell::new(a.notes.as_deref().unwrap_or("")).fg(Color::DarkGrey),
            ]);
        }
    } else {
        table.set_header(vec![
            Cell::new("Date").add_attribute(Attribute::Bold),
            Cell::new("Time").add_attribute(Attribute::Bold),
            Cell::new("Difficulty").add_attribute(Attribute::Bold),
            Cell::new("Topic").add_attribute(Attribute::Bold),
            Cell::new("Notes").add_attribute(Attribute::Bold),
        ]);
        for a in activities {
            let diff = a.lc_difficulty.as_deref().unwrap_or("–");
            let diff_cell = match diff {
                "Easy"   => Cell::new(diff).fg(Color::Green),
                "Medium" => Cell::new(diff).fg(Color::Yellow),
                "Hard"   => Cell::new(diff).fg(Color::Red),
                _        => Cell::new(diff),
            };
            table.add_row(vec![
                Cell::new(&a.date),
                Cell::new(a.time.as_deref().unwrap_or("–")),
                diff_cell,
                Cell::new(a.topic.as_deref().unwrap_or("")).fg(Color::DarkGrey),
                Cell::new(a.notes.as_deref().unwrap_or("")).fg(Color::DarkGrey),
            ]);
        }
    }

    for line in table.to_string().lines() {
        println!("    {}", line);
    }
    println!();
}
