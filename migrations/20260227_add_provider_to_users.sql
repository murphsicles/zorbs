-- Add multi-provider support for users
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS provider TEXT NOT NULL DEFAULT 'github',
ADD COLUMN IF NOT EXISTS provider_id TEXT;

-- Migrate existing GitHub users
UPDATE users 
SET provider_id = github_id::TEXT 
WHERE github_id IS NOT NULL AND provider_id IS NULL;

-- Make github_id optional for future
ALTER TABLE users ALTER COLUMN github_id DROP NOT NULL;
