CREATE TABLE IF NOT EXISTS collector_snapshots (
    id BIGSERIAL PRIMARY KEY,
    target_url VARCHAR(2048) NOT NULL,
    captured_at TIMESTAMPTZ NOT NULL,
    snapshot JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_collector_snapshots_target_time
    ON collector_snapshots (target_url, captured_at DESC, id DESC);

CREATE TABLE IF NOT EXISTS exploit_rate_limits (
    target_domain VARCHAR(512) NOT NULL,
    window_start TIMESTAMPTZ NOT NULL,
    request_count INTEGER NOT NULL CHECK (request_count >= 0),
    PRIMARY KEY (target_domain, window_start)
);

CREATE INDEX IF NOT EXISTS idx_exploit_rate_limits_window
    ON exploit_rate_limits (window_start);
