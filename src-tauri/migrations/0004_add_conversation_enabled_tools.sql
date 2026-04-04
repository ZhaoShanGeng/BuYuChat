-- 为会话添加已启用工具列表字段。
-- JSON array string, e.g. '["fetch"]'. NULL 表示使用全局默认。
ALTER TABLE conversations ADD COLUMN enabled_tools TEXT DEFAULT NULL;
