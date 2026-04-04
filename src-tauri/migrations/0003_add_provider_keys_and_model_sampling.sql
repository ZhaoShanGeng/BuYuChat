ALTER TABLE api_channels ADD COLUMN api_keys TEXT;

ALTER TABLE api_channel_models ADD COLUMN temperature TEXT;
ALTER TABLE api_channel_models ADD COLUMN top_p TEXT;
