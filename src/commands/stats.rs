use anyhow::{Context, Result};
use sqlx::SqlitePool;

// ─── Stats data structures 

#[derive(Debug, Default)]
pub struct CfStats {
    pub total:        i64,
    pub avg_rating:   f64,
    pub this_week:    i64,
    pub this_month:   i64,
    // Rating buckets: <1200, 1200-1399, 1400-1599, 1600-1799, 1800-1999, 2000-2199, ≥2200
    pub rating_buckets: [i64; 7],
    // Difficulty counts: index 0 = diff 1 (Easy) … index 4 = diff 5 (Insane)
    pub difficulty_counts: [i64; 5],
    // Top tags: (tag, count)
    pub top_tags:     Vec<(String, usize)>,
}

#[derive(Debug, Default)]
pub struct LcStats {
    pub total:      i64,
    pub easy:       i64,
    pub medium:     i64,
    pub hard:       i64,
    pub this_week:  i64,
    pub this_month: i64,
    // Top topics: (topic, count)
    pub top_topics: Vec<(String, usize)>,
}

#[derive(Debug, Default)]
pub struct NoteStats {
    pub total:      i64,
    pub this_week:  i64,
    pub this_month: i64,
}

#[derive(Debug, Default)]
pub struct OverallStats {
    pub active_days:     i64,
    pub first_log_date:  Option<String>,
    pub current_streak:  i64,
}

#[derive(Debug, Default)]
pub struct StatsData {
    pub cf:         CfStats,
    pub lc:         LcStats,
    pub notes:      NoteStats,
    pub overall:    OverallStats,
    pub days_filter: Option<u32>,
}

// ─── Fetch ─────────

pub async fn fetch_stats(pool: &SqlitePool, days: Option<u32>) -> Result<StatsData> {
    let mut data = StatsData { days_filter: days, ..Default::default() };

    // Build the date cutoff clause
    let date_clause_activities = match days {
        Some(n) => format!(
            "AND date >= date('now', '-{} days')",
            n
        ),
        None => String::new(),
    };
    let date_clause_notes = date_clause_activities.clone();

    // ── Codeforces ─

    let cf_rows: Vec<(Option<i64>, Option<i64>, Option<String>)> = sqlx::query_as(
        &format!(
            "SELECT rating, difficulty, tags FROM activities
             WHERE platform = 'codeforces' {}",
            date_clause_activities
        )
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch CF stats")?;

    let cf_week: (i64,) = sqlx::query_as(
        &format!(
            "SELECT COUNT(*) FROM activities
             WHERE platform='codeforces' AND date >= date('now','-7 days') {}",
            if days.is_some() { &date_clause_activities } else { "" }
        )
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch CF this_week")?;

    let cf_month: (i64,) = sqlx::query_as(
        &format!(
            "SELECT COUNT(*) FROM activities
             WHERE platform='codeforces' AND date >= date('now','-30 days') {}",
            if days.is_some() { &date_clause_activities } else { "" }
        )
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch CF this_month")?;

    data.cf.this_week  = cf_week.0;
    data.cf.this_month = cf_month.0;

    let mut total_rating = 0i64;
    let mut rated_count  = 0i64;
    let mut tag_freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for (rating, difficulty, tags_json) in &cf_rows {
        data.cf.total += 1;

        if let Some(r) = rating {
            total_rating += r;
            rated_count  += 1;
            let bucket = match r {
                r if *r < 1200 => 0,
                r if *r < 1400 => 1,
                r if *r < 1600 => 2,
                r if *r < 1800 => 3,
                r if *r < 2000 => 4,
                r if *r < 2200 => 5,
                _              => 6,
            };
            data.cf.rating_buckets[bucket] += 1;
        }

        if let Some(d) = difficulty {
            let idx = ((*d).clamp(1, 5) - 1) as usize;
            data.cf.difficulty_counts[idx] += 1;
        }

        // Parse tags JSON array
        if let Some(tj) = tags_json {
            if let Ok(tags) = serde_json::from_str::<Vec<String>>(tj) {
                for tag in tags {
                    *tag_freq.entry(tag).or_insert(0) += 1;
                }
            }
        }
    }

    if rated_count > 0 {
        data.cf.avg_rating = total_rating as f64 / rated_count as f64;
    }

    // Top 5 tags
    let mut tag_vec: Vec<(String, usize)> = tag_freq.into_iter().collect();
    tag_vec.sort_by(|a, b| b.1.cmp(&a.1));
    data.cf.top_tags = tag_vec.into_iter().take(5).collect();

    // ── LeetCode ───

    let lc_rows: Vec<(Option<String>, Option<String>)> = sqlx::query_as(
        &format!(
            "SELECT lc_difficulty, topic FROM activities
             WHERE platform = 'leetcode' {}",
            date_clause_activities
        )
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch LC stats")?;

    let lc_week: (i64,) = sqlx::query_as(
        &format!(
            "SELECT COUNT(*) FROM activities
             WHERE platform='leetcode' AND date >= date('now','-7 days') {}",
            if days.is_some() { &date_clause_activities } else { "" }
        )
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch LC this_week")?;

    let lc_month: (i64,) = sqlx::query_as(
        &format!(
            "SELECT COUNT(*) FROM activities
             WHERE platform='leetcode' AND date >= date('now','-30 days') {}",
            if days.is_some() { &date_clause_activities } else { "" }
        )
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch LC this_month")?;

    data.lc.this_week  = lc_week.0;
    data.lc.this_month = lc_month.0;

    let mut topic_freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for (lc_diff, topic) in &lc_rows {
        data.lc.total += 1;
        match lc_diff.as_deref() {
            Some("Easy")   => data.lc.easy   += 1,
            Some("Medium") => data.lc.medium += 1,
            Some("Hard")   => data.lc.hard   += 1,
            _              => {}
        }
        if let Some(t) = topic {
            if !t.is_empty() {
                *topic_freq.entry(t.clone()).or_insert(0) += 1;
            }
        }
    }

    // Top 5 topics
    let mut topic_vec: Vec<(String, usize)> = topic_freq.into_iter().collect();
    topic_vec.sort_by(|a, b| b.1.cmp(&a.1));
    data.lc.top_topics = topic_vec.into_iter().take(5).collect();

    // ── Notes ──────

    let note_total: (i64,) = sqlx::query_as(
        &format!(
            "SELECT COUNT(*) FROM notes WHERE 1=1 {}",
            date_clause_notes
        )
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch note count")?;

    let note_week: (i64,) = sqlx::query_as(
        &format!(
            "SELECT COUNT(*) FROM notes WHERE date >= date('now','-7 days') {}",
            if days.is_some() { &date_clause_notes } else { "" }
        )
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch note this_week")?;

    let note_month: (i64,) = sqlx::query_as(
        &format!(
            "SELECT COUNT(*) FROM notes WHERE date >= date('now','-30 days') {}",
            if days.is_some() { &date_clause_notes } else { "" }
        )
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch note this_month")?;

    data.notes.total      = note_total.0;
    data.notes.this_week  = note_week.0;
    data.notes.this_month = note_month.0;

    // ── Overall ────

    // All distinct active dates (union of activities + notes)
    let active_dates: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT date FROM activities
         UNION
         SELECT DISTINCT date FROM notes
         ORDER BY date ASC"
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch active dates")?;

    data.overall.active_days   = active_dates.len() as i64;
    data.overall.first_log_date = active_dates.first().map(|(d,)| d.clone());

    // Streak: count consecutive days ending today or yesterday
    let streak = compute_streak(&active_dates.iter().map(|(d,)| d.as_str()).collect::<Vec<_>>());
    data.overall.current_streak = streak;

    Ok(data)
}

// ─── Streak helper ─

fn compute_streak(dates: &[&str]) -> i64 {
    use chrono::{Duration, Local, NaiveDate};

    if dates.is_empty() {
        return 0;
    }

    let today     = Local::now().date_naive();
    let yesterday = today - Duration::days(1);

    // Parse all dates into a set
    let date_set: std::collections::HashSet<NaiveDate> = dates
        .iter()
        .filter_map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .collect();

    // Start from today or yesterday
    let start = if date_set.contains(&today) {
        today
    } else if date_set.contains(&yesterday) {
        yesterday
    } else {
        return 0;
    };

    let mut streak = 0i64;
    let mut cur = start;
    loop {
        if date_set.contains(&cur) {
            streak += 1;
            cur = cur - Duration::days(1);
        } else {
            break;
        }
    }

    streak
}
