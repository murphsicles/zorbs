CREATE TABLE IF NOT EXISTS zorbs (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    description TEXT,
    license TEXT,
    repository TEXT,
    downloads BIGINT DEFAULT 0 NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    UNIQUE (name, version)
);

CREATE INDEX IF NOT EXISTS idx_zorbs_downloads ON zorbs (downloads DESC);
CREATE INDEX IF NOT EXISTS idx_zorbs_name ON zorbs (name);
ALTER TABLE zorbs ADD COLUMN IF NOT EXISTS owner_id UUID REFERENCES users(id);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    data JSONB NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);
