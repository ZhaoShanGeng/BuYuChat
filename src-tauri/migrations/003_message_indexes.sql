CREATE INDEX IF NOT EXISTS idx_messages_parent_active_created
    ON messages(parent_message_id, is_active, created_at ASC);

CREATE INDEX IF NOT EXISTS idx_messages_conv_parent_active_created
    ON messages(conversation_id, parent_message_id, is_active, created_at ASC);

CREATE INDEX IF NOT EXISTS idx_messages_vgroup_active_version
    ON messages(version_group_id, is_active, version_index DESC);
