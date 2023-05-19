create table dns_records (
    query_type TINYINT NOT NULL,
    domain TEXT NOT NULL,
    host TEXT NOT NULL,
    ttl INTEGER NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (query_type, domain)
);
