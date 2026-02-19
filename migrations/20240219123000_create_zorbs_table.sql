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
