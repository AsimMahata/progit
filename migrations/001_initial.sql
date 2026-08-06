-- Migration 001: Initial schema

-- Unified activity table for Codeforces, LeetCode, and any future platform.
-- Platform-specific fields are left NULL for platforms that don't use them.
CREATE TABLE IF NOT EXISTS activities (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    platform      TEXT    NOT NULL,
    date          TEXT    NOT NULL,  -- YYYY-MM-DD
    time          TEXT,              -- HH:MM  (nullable)
    difficulty    INTEGER,           -- 1–5 unified scale
    rating        INTEGER,           -- nullable (Codeforces)
    lc_difficulty TEXT,              -- nullable (LeetCode: Easy/Medium/Hard)
    topic         TEXT,              -- nullable (LeetCode topic, CF category, etc.)
    tags          TEXT,              -- nullable JSON array e.g. ["E1","WA3","BinarySearch"]
    notes         TEXT,              -- nullable free text
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Trigger to auto-update updated_at on activities
CREATE TRIGGER IF NOT EXISTS activities_updated_at
    AFTER UPDATE ON activities
    FOR EACH ROW
    BEGIN
        UPDATE activities SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
        WHERE id = OLD.id;
    END;

-- Tasks table: separate lifecycle from activities.
-- status is nullable, fully custom (Todo/Doing/Done/Cancelled are suggestions not enforced).
-- priority: 1=Low (default), 2=Medium, 3=High.
CREATE TABLE IF NOT EXISTS tasks (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    title       TEXT    NOT NULL,
    description TEXT,
    status      TEXT,               -- nullable, free-form
    priority    INTEGER NOT NULL DEFAULT 1,  -- 1=Low 2=Medium 3=High
    deadline    TEXT,               -- nullable YYYY-MM-DD
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Trigger to auto-update updated_at on tasks
CREATE TRIGGER IF NOT EXISTS tasks_updated_at
    AFTER UPDATE ON tasks
    FOR EACH ROW
    BEGIN
        UPDATE tasks SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
        WHERE id = OLD.id;
    END;

-- Quick freeform log entries.
CREATE TABLE IF NOT EXISTS notes (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    date       TEXT NOT NULL,  -- YYYY-MM-DD
    time       TEXT,           -- HH:MM (nullable)
    text       TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Trigger to auto-update updated_at on notes
CREATE TRIGGER IF NOT EXISTS notes_updated_at
    AFTER UPDATE ON notes
    FOR EACH ROW
    BEGIN
        UPDATE notes SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
        WHERE id = OLD.id;
    END;
