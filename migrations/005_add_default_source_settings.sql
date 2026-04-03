-- Migration: Add default source settings table
-- Allows admins to control which stream source is used by default

CREATE TABLE IF NOT EXISTS default_source_settings (
    id SERIAL PRIMARY KEY,
    source_name VARCHAR(50) UNIQUE NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    priority INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Insert default sources (these can be updated by admins)
INSERT INTO default_source_settings (source_name, is_default, priority, is_active)
VALUES 
    ('source1', TRUE, 1, TRUE),
    ('source2', FALSE, 2, TRUE),
    ('source3', FALSE, 3, TRUE)
ON CONFLICT (source_name) DO NOTHING;

-- Index for quick default source lookup
CREATE INDEX IF NOT EXISTS idx_default_source_active ON default_source_settings(is_active, is_default, priority);
