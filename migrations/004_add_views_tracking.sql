-- Create match views table for IP tracking
CREATE TABLE IF NOT EXISTS match_views (
  id SERIAL PRIMARY KEY,
  match_id VARCHAR(255) NOT NULL,
  ip_address VARCHAR(45) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(match_id, ip_address)
);

-- Add views column to matches table if not exists
ALTER TABLE matches ADD COLUMN IF NOT EXISTS views INTEGER DEFAULT 0;

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_match_views_match_id ON match_views(match_id);
CREATE INDEX IF NOT EXISTS idx_match_views_ip ON match_views(ip_address);
