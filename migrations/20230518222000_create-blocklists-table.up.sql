create table blocklists (
    id INTEGER NOT NULL PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL,
    last_refresh_at INTEGER,
    last_refresh_hash TEXT
);