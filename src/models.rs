
// ─── Activity (Codeforces / LeetCode / future platforms) ─────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Activity {
    pub id:            i64,
    pub platform:      String,
    pub date:          String,
    pub time:          Option<String>,
    pub difficulty:    Option<i64>,
    pub rating:        Option<i64>,
    pub lc_difficulty: Option<String>,
    pub topic:         Option<String>,
    pub tags:          Option<String>, // JSON array stored as text
    pub notes:         Option<String>,
    pub created_at:    String,
    pub updated_at:    String,
}

impl Activity {
    /// Deserialize the JSON tags field into a Vec<String>.
    pub fn parse_tags(&self) -> Vec<String> {
        self.tags
            .as_deref()
            .and_then(|t| serde_json::from_str(t).ok())
            .unwrap_or_default()
    }

    /// Difficulty as a star string: e.g. 3 → "★★★☆☆"
    pub fn difficulty_stars(&self) -> String {
        match self.difficulty {
            Some(d) => {
                let d = d.clamp(1, 5) as usize;
                format!("{}{}", "★".repeat(d), "☆".repeat(5 - d))
            }
            None => "–".to_string(),
        }
    }

    /// Return the LC difficulty string if present (for display in edit confirmation).
    pub fn lc_diff_display(&self) -> Option<String> {
        self.lc_difficulty.clone()
    }
}

// ─── Task ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Task {
    pub id:          i64,
    pub title:       String,
    pub description: Option<String>,
    pub status:      Option<String>,
    pub priority:    i64, // 1=Low 2=Medium 3=High
    pub deadline:    Option<String>,
    pub created_at:  String,
    pub updated_at:  String,
}

impl Task {
    pub fn priority_label(&self) -> &'static str {
        match self.priority {
            2 => "Medium",
            3 => "High",
            _ => "Low",
        }
    }

    pub fn status_display(&self) -> &str {
        self.status.as_deref().unwrap_or("–")
    }
}

/// Parse a priority value from user input: "1"/"low" → 1, "2"/"medium" → 2, "3"/"high" → 3.
pub fn parse_priority(s: &str) -> Option<i64> {
    match s.trim().to_lowercase().as_str() {
        "1" | "low"    => Some(1),
        "2" | "medium" => Some(2),
        "3" | "high"   => Some(3),
        _              => None,
    }
}

// ─── Note ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Note {
    pub id:         i64,
    pub date:       String,
    pub time:       Option<String>,
    pub text:       String,
    pub created_at: String,
    pub updated_at: String,
}

// ─── DayView ─────────────────────────────────────────────────────────────────

/// Everything logged on a given date, collected for display.
pub struct DayView {
    pub date:       String,
    pub activities: Vec<Activity>,
    pub tasks:      Vec<Task>,
    pub notes:      Vec<Note>,
}
