use clap::{Parser, Subcommand};

// ─── Top-level CLI ─

#[derive(Parser)]
#[command(
    name = "progit",
    about = "Your personal engineering journal",
    long_about = "Progit — a lightweight CLI for tracking daily technical progress.\n\
                  Not a task manager. Not a habit tracker. Just a fast engineering journal.",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

// ─── Subcommands ──

#[derive(Subcommand)]
pub enum Commands {
    /// Log a Codeforces problem  [alias: cf]
    ///
    /// Examples:
    ///   progit cf 1700 1 E1
    ///   progit cf 1900 3 E2 T5
    ///   progit cf 2100 5 CHT "Needed editorial"
    ///   progit cf help
    #[command(name = "cf", alias = "codeforces")]
    Cf {
        /// Show usage help for the cf command
        #[arg(long, action = clap::ArgAction::SetTrue)]
        help_me: bool,

        /// Problem rating (e.g. 1700, 1900)
        rating: Option<i64>,

        /// Difficulty from 1 (Easy) to 5 (Insane)
        difficulty: Option<i64>,

        /// Tags (bare tokens) and/or a quoted note string
        /// Examples: E1 WA3 BinarySearch "Forgot binary search"
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        rest: Vec<String>,

        /// Override date (YYYY-MM-DD), defaults to today
        #[arg(long, short = 'd')]
        date: Option<String>,

        /// Override time (HH:MM), defaults to now
        #[arg(long, short = 't')]
        time: Option<String>,
    },

    /// Log a LeetCode problem  [alias: lc]
    ///
    /// Examples:
    ///   progit lc Easy
    ///   progit lc Hard Graph
    ///   progit lc Medium DP "Needed hints"
    ///   progit lc help
    #[command(name = "lc", alias = "leetcode")]
    Lc {
        /// Difficulty: Easy, Medium, or Hard
        difficulty: Option<String>,

        /// Optional topic (e.g. Graph, DP, SegmentTree)
        topic: Option<String>,

        /// Optional notes (quoted string)
        notes: Option<String>,

        /// Override date (YYYY-MM-DD), defaults to today
        #[arg(long, short = 'd')]
        date: Option<String>,

        /// Override time (HH:MM), defaults to now
        #[arg(long, short = 't')]
        time: Option<String>,
    },

    /// Manage tasks
    ///
    /// Examples:
    ///   progit task add
    ///   progit task edit 3
    ///   progit task status 3 Doing
    #[command(name = "task")]
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },

    /// List tasks (shorthand for `task list`)
    ///
    /// Examples:
    ///   progit tasks
    ///   progit tasks --doing
    ///   progit tasks --done
    #[command(name = "tasks")]
    Tasks {
        /// Show only Todo tasks
        #[arg(long, conflicts_with_all = &["doing", "done", "cancelled"])]
        todo: bool,

        /// Show only Doing tasks
        #[arg(long, conflicts_with_all = &["todo", "done", "cancelled"])]
        doing: bool,

        /// Show only Done tasks
        #[arg(long, conflicts_with_all = &["todo", "doing", "cancelled"])]
        done: bool,

        /// Show only Cancelled tasks
        #[arg(long, conflicts_with_all = &["todo", "doing", "done"])]
        cancelled: bool,
    },

    /// List Codeforces entries  (progit cf list)
    ///
    /// Example: progit cf list
    ///          progit cf list --all
    #[command(name = "cf-list", hide = true)]
    CfList {
        /// Show all entries (default: last 20)
        #[arg(long)]
        all: bool,
    },

    /// List LeetCode entries  (progit lc list)
    ///
    /// Example: progit lc list
    ///          progit lc list --all
    #[command(name = "lc-list", hide = true)]
    LcList {
        /// Show all entries (default: last 20)
        #[arg(long)]
        all: bool,
    },

    /// Log a quick note
    ///
    /// Examples:
    ///   progit note "Groww OA Rejected"
    ///   progit note "Learnt CHT"
    #[command(name = "note")]
    Note {
        /// The note text
        text: String,

        /// Override date (YYYY-MM-DD), defaults to today
        #[arg(long, short = 'd')]
        date: Option<String>,

        /// Override time (HH:MM), defaults to now
        #[arg(long, short = 't')]
        time: Option<String>,
    },

    /// Show today's activity
    #[command(name = "today")]
    Today,

    /// Show yesterday's activity
    #[command(name = "yesterday")]
    Yesterday,

    /// Show activity for a specific date (YYYY-MM-DD)
    ///
    /// Example: progit date 2026-08-01
    #[command(name = "date")]
    Date {
        /// Date in YYYY-MM-DD format
        date: String,
    },

    /// Show activity for the last N days
    ///
    /// Example: progit last 7
    #[command(name = "last")]
    Last {
        /// Number of days
        days: u32,
    },

    /// Edit a Codeforces or LeetCode activity by ID
    ///
    /// Pass only the fields you want to change — everything else stays as-is.
    ///
    /// Examples:
    ///   progit edit 3 --notes "Needed editorial"
    ///   progit edit 5 --rating 1900 --difficulty 3
    ///   progit edit 7 --date 2026-08-01 --tags "E1,WA3,BinarySearch"
    #[command(name = "edit")]
    Edit {
        /// Activity ID (shown in cf list / lc list / today output)
        id: i64,

        /// New date (YYYY-MM-DD)
        #[arg(long, short = 'd')]
        date: Option<String>,

        /// New time (HH:MM)
        #[arg(long, short = 't')]
        time: Option<String>,

        /// New rating (Codeforces)
        #[arg(long, short = 'r')]
        rating: Option<i64>,

        /// New difficulty (1–5)
        #[arg(long)]
        difficulty: Option<i64>,

        /// New notes (replaces existing)
        #[arg(long, short = 'n')]
        notes: Option<String>,

        /// New tags — comma-separated (replaces existing)
        /// Example: --tags "E1,WA3,BinarySearch"
        #[arg(long)]
        tags: Option<String>,

        /// New topic (LeetCode)
        #[arg(long)]
        topic: Option<String>,

        /// New LeetCode difficulty (Easy | Medium | Hard)
        #[arg(long)]
        lc_difficulty: Option<String>,
    },

    /// Edit a note by ID
    ///
    /// Examples:
    ///   progit edit-note 2 --text "GROWw OA Rejected"
    ///   progit edit-note 2 --date 2026-08-04
    #[command(name = "edit-note")]
    EditNote {
        /// Note ID (shown in today / yesterday / date output)
        id: i64,

        /// New note text
        #[arg(long)]
        text: Option<String>,

        /// New date (YYYY-MM-DD)
        #[arg(long, short = 'd')]
        date: Option<String>,

        /// New time (HH:MM)
        #[arg(long, short = 't')]
        time: Option<String>,
    },

    /// Uninstall progit — removes the binary and all data in ~/.progit/
    #[command(name = "uninstall")]
    Uninstall,

    /// Show overall progress stats (notes, Codeforces, LeetCode)
    ///
    /// Examples:
    ///   progit stats
    ///   progit stats --days 30
    ///   progit stats --days 7
    #[command(name = "stats")]
    Stats {
        /// Narrow stats to the last N days (default: all time)
        #[arg(long, short = 'd')]
        days: Option<u32>,
    },

    /// Create a full database backup checkpoint
    ///
    /// Saves all data to ~/tools/progit/backups/<name>/
    /// and updates ~/tools/progit/current/ with the latest snapshot.
    ///
    /// Examples:
    ///   progit backup
    ///   progit backup pre-hackathon
    ///   progit backup 2026-08-07
    #[command(name = "backup")]
    Backup {
        /// Checkpoint name (default: today's date YYYY-MM-DD)
        name: Option<String>,
    },

    /// Restore database from a named backup checkpoint
    ///
    /// Will ask before overwriting your current data.
    ///
    /// Examples:
    ///   progit restore 2026-08-07
    ///   progit restore pre-hackathon
    #[command(name = "restore")]
    Restore {
        /// Checkpoint name to restore from
        name: String,
    },

    /// List all available backup checkpoints
    ///
    /// Example: progit backups
    #[command(name = "backups")]
    Backups,
}

// ─── Task subcommands ──────

#[derive(Subcommand)]
pub enum TaskAction {
    /// Add a new task (interactive prompts)
    Add,

    /// Edit an existing task by ID (interactive)
    Edit {
        /// Task ID
        id: i64,
    },

    /// Update the status of a task
    ///
    /// Example: progit task status 3 Doing
    Status {
        /// Task ID
        id: i64,
        /// New status (any string, e.g. Todo, Doing, Done, Cancelled)
        status: String,
    },

    /// List tasks
    List {
        /// Filter by status
        #[arg(long)]
        filter: Option<String>,
    },
}
