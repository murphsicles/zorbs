-- Add dependencies JSONB and readme TEXT columns to zorbs table
ALTER TABLE zorbs ADD COLUMN IF NOT EXISTS dependencies JSONB DEFAULT '{}'::jsonb;
ALTER TABLE zorbs ADD COLUMN IF NOT EXISTS readme TEXT;
