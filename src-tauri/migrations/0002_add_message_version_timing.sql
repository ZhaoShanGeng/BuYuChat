ALTER TABLE message_versions ADD COLUMN received_at INTEGER;
ALTER TABLE message_versions ADD COLUMN completed_at INTEGER;

UPDATE message_versions
SET received_at = created_at
WHERE received_at IS NULL;
