create table blocked_domains (
    id INTEGER NOT NULL PRIMARY KEY,
    blocklist_id INTEGER REFERENCES blocklists(id) ON DELETE CASCADE,
    domain TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE (blocklist_id, domain)
);