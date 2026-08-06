# Progit

> **Your personal engineering journal — built for speed, not analytics.**

Progit is a lightweight CLI that answers one question at the end of the day:
*"What did I actually do today?"*

It is **not** a task manager. It is **not** a habit tracker.  
It is a fast engineering journal you log to in seconds, straight from the terminal.

---

## What it tracks

| What                    | Command             |
|-------------------------|---------------------|
| Codeforces problems     | `progit cf`         |
| LeetCode problems       | `progit lc`         |
| Tasks (Todo/Doing/Done) | `progit task`       |
| Quick notes             | `progit note`       |
| Today's summary         | `progit today`      |
| Last N days             | `progit last <N>`   |

---

## Installation

### Prerequisites
- Rust toolchain (`rustup` recommended): https://rustup.rs

### Build from source

```bash
git clone https://github.com/yourname/progit
cd progit
cargo build --release
```

The binary will be at `target/release/progit`.

To install it globally:

```bash
cargo install --path .
```

### Data storage

All data is stored locally at:

```
~/.progit/
└── database.db
```

No cloud. No sync. Yours.

---

## Usage

### Log a Codeforces problem

```bash
progit cf <rating> <difficulty> [tags...] ["notes"]
```

| Argument     | Description                                    |
|--------------|------------------------------------------------|
| `rating`     | Problem rating — e.g. `1700`, `1900`, `2100`   |
| `difficulty` | `1` Easy · `2` Medium · `3` Hard · `4` Very Hard · `5` Insane |
| `tags`       | Any bare tokens: `E1 WA3 BinarySearch CHT` etc. |
| `notes`      | Quoted string at the end (optional)            |

**Examples**

```bash
progit cf 1700 1 E1
progit cf 1900 3 E2 T5
progit cf 2100 5 CHT "Needed editorial"
progit cf 1800 3 WA3 BinarySearch "Off-by-one in binary search"
progit cf 1600 2 WA3 BinarySearch
```

**Override date / time**

```bash
progit cf 1700 2 E1 --date 2026-08-01 --time 09:30
```

**Show help**

```bash
progit cf help
```

**List recent entries**

```bash
progit cf list          # last 20
progit cf list --all    # all time
```

---

### Log a LeetCode problem

```bash
progit lc <Easy|Medium|Hard> [topic] ["notes"]
```

**Examples**

```bash
progit lc Easy
progit lc Hard Graph
progit lc Medium DP
progit lc Hard SegmentTree "Needed hints"
```

**Show help**

```bash
progit lc help
```

**List recent entries**

```bash
progit lc list          # last 20
progit lc list --all    # all time
```

---

### Manage tasks

#### Add a task (interactive)

```bash
progit task add
```

You'll be prompted for:
- **Title** (required)
- **Description** (optional)
- **Status** — any string, e.g. `Todo`, `Doing`, `Done`, `Cancelled`
- **Priority** — `1` Low (default), `2` Medium, `3` High
- **Deadline** — `YYYY-MM-DD` (optional)

#### Edit a task

```bash
progit task edit <id>
```

Press Enter to keep the current value for any field. Type `-` to clear an optional field.

#### Update status

```bash
progit task status <id> <status>
```

Status is free-form — any string works.

```bash
progit task status 3 Doing
progit task status 5 "In Review"
progit task status 7 Done
```

#### List tasks

```bash
progit tasks               # all tasks
progit tasks --todo        # only Todo
progit tasks --doing       # only Doing
progit tasks --done        # only Done
progit tasks --cancelled   # only Cancelled
```

---

### Log a quick note

```bash
progit note "Groww OA Rejected"
progit note "Learnt CHT"
progit note "Started studying TLS"
progit note "Passport received"
```

Override date/time:

```bash
progit note "Resolved prod incident" --date 2026-08-05 --time 23:45
```

---

### View your activity

#### Today

```bash
progit today
```

Output:

```
  ════════════════════════════════════════
  📅  06 August 2026
  ════════════════════════════════════════

  ⚡ CODEFORCES
  ────────────────────────────────────────
    ┌───────┬────────┬───────────┬─────────────────┬────────────────┐
    │ Time  │ Rating │ Difficulty│ Tags            │ Notes          │
    ├───────┼────────┼───────────┼─────────────────┼────────────────┤
    │ 09:15 │ 1700   │ ★☆☆☆☆    │ E1              │                │
    └───────┴────────┴───────────┴─────────────────┴────────────────┘

  📘 LEETCODE
  ────────────────────────────────────────
    ┌───────┬────────┬──────┬──────────────┐
    │ Time  │ Diff   │ Topic│ Notes        │
    ├───────┼────────┼──────┼──────────────┤
    │ 10:40 │ Medium │ DP   │              │
    └───────┴────────┴──────┴──────────────┘

  ✅ TASKS
  ────────────────────────────────────────
    ┌────┬────────────────────────┬───────┬──────────┬──────────┐
    │ ID │ Title                  │ Status│ Priority │ Deadline │
    ├────┼────────────────────────┼───────┼──────────┼──────────┤
    │  1 │ Finish Dusty Refactor  │ Doing │ High     │          │
    └────┴────────────────────────┴───────┴──────────┴──────────┘

  📝 NOTES
  ────────────────────────────────────────
    [09:00] • Started TLS
```

#### Yesterday

```bash
progit yesterday
```

#### Specific date

```bash
progit date 2026-08-01
```

#### Last N days

```bash
progit last 7
progit last 30
```

---

## Difficulty scale (Codeforces)

| Value | Meaning   |
|-------|-----------|
| 1     | Easy      |
| 2     | Medium    |
| 3     | Hard      |
| 4     | Very Hard |
| 5     | Insane    |

Displayed as stars: `★★★☆☆`

---

## Tag system

Tags are completely free-form. Progit never validates or restricts tags.

**Common conventions** (not enforced):

| Tag          | Meaning                    |
|--------------|----------------------------|
| `E1`, `E2`   | Editorial read attempt     |
| `T1`, `T2`   | Time to solve (in hours)   |
| `WA1`, `WA3` | Wrong answer attempts      |
| `MLE`, `TLE` | Memory / Time limit errors |
| `BinarySearch`, `CHT`, `DP` | Algorithm tags |

---

## Adding a new platform (for contributors)

Progit uses a single unified `activities` table. Adding a new platform requires:

1. A new command handler in `src/commands/<platform>.rs`
2. A new `Commands::` variant in `src/cli.rs`
3. A match arm in `src/main.rs`

No new database tables are needed.

---

## Tech stack

| Crate          | Purpose                        |
|----------------|--------------------------------|
| `clap`         | CLI argument parsing           |
| `sqlx`         | Async SQLite with compile-time queries |
| `tokio`        | Async runtime                  |
| `chrono`       | Date/time handling             |
| `serde_json`   | Tag serialization              |
| `comfy-table`  | Pretty terminal tables         |
| `owo-colors`   | Terminal colors                |
| `dirs`         | Cross-platform home dir        |
| `anyhow`       | Error handling                 |

---

## License

MIT
