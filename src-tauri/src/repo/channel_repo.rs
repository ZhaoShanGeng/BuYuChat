use rusqlite::{params, Connection, OptionalExtension};

use crate::models::{Channel, ChannelPatch, NewChannel};

pub trait ChannelRepo {
    fn insert(&self, new_channel: &NewChannel) -> Result<Channel, String>;
    fn list(&self, include_disabled: bool) -> Result<Vec<Channel>, String>;
    fn get(&self, id: &str) -> Result<Option<Channel>, String>;
    fn update(&self, id: &str, patch: &ChannelPatch) -> Result<Option<Channel>, String>;
    fn delete(&self, id: &str) -> Result<bool, String>;
}

pub struct SqliteChannelRepo<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteChannelRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

fn row_to_channel(row: &rusqlite::Row<'_>) -> rusqlite::Result<Channel> {
    Ok(Channel {
        id: row.get("id")?,
        name: row.get("name")?,
        channel_type: row.get("channel_type")?,
        base_url: row.get("base_url")?,
        api_key: row.get("api_key")?,
        auth_type: row.get("auth_type")?,
        models_endpoint: row.get("models_endpoint")?,
        chat_endpoint: row.get("chat_endpoint")?,
        stream_endpoint: row.get("stream_endpoint")?,
        enabled: row.get("enabled")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn insert(conn: &Connection, new_channel: &NewChannel) -> rusqlite::Result<Channel> {
    conn.execute(
        r#"
        INSERT INTO api_channels (
            id, name, channel_type, base_url, api_key, auth_type,
            models_endpoint, chat_endpoint, stream_endpoint, enabled, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        params![
            new_channel.id,
            new_channel.name,
            new_channel.channel_type,
            new_channel.base_url,
            new_channel.api_key,
            new_channel.auth_type,
            new_channel.models_endpoint,
            new_channel.chat_endpoint,
            new_channel.stream_endpoint,
            new_channel.enabled,
            new_channel.created_at,
            new_channel.updated_at
        ],
    )?;

    get(conn, &new_channel.id).map(|channel| channel.expect("inserted channel must exist"))
}

pub fn list(conn: &Connection, include_disabled: bool) -> rusqlite::Result<Vec<Channel>> {
    let mut stmt = if include_disabled {
        conn.prepare(
            r#"
            SELECT * FROM api_channels
            ORDER BY created_at DESC, id DESC
            "#,
        )?
    } else {
        conn.prepare(
            r#"
            SELECT * FROM api_channels
            WHERE enabled = 1
            ORDER BY created_at DESC, id DESC
            "#,
        )?
    };

    let rows = stmt.query_map([], row_to_channel)?;
    rows.collect()
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Channel>> {
    conn.query_row(
        "SELECT * FROM api_channels WHERE id = ?",
        params![id],
        row_to_channel,
    )
    .optional()
}

pub fn update(
    conn: &Connection,
    id: &str,
    patch: &ChannelPatch,
) -> rusqlite::Result<Option<Channel>> {
    let changed = conn.execute(
        r#"
        UPDATE api_channels
        SET
            name = COALESCE(?1, name),
            base_url = COALESCE(?2, base_url),
            channel_type = COALESCE(?3, channel_type),
            api_key = COALESCE(?4, api_key),
            auth_type = COALESCE(?5, auth_type),
            models_endpoint = COALESCE(?6, models_endpoint),
            chat_endpoint = COALESCE(?7, chat_endpoint),
            stream_endpoint = COALESCE(?8, stream_endpoint),
            enabled = COALESCE(?9, enabled),
            updated_at = ?10
        WHERE id = ?11
        "#,
        params![
            patch.name,
            patch.base_url,
            patch.channel_type,
            patch.api_key,
            patch.auth_type,
            patch.models_endpoint,
            patch.chat_endpoint,
            patch.stream_endpoint,
            patch.enabled,
            patch.updated_at,
            id
        ],
    )?;

    if changed == 0 {
        return Ok(None);
    }

    get(conn, id)
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<bool> {
    let changed = conn.execute("DELETE FROM api_channels WHERE id = ?", params![id])?;
    Ok(changed > 0)
}

impl ChannelRepo for SqliteChannelRepo<'_> {
    fn insert(&self, new_channel: &NewChannel) -> Result<Channel, String> {
        insert(self.conn, new_channel).map_err(|err| err.to_string())
    }

    fn list(&self, include_disabled: bool) -> Result<Vec<Channel>, String> {
        list(self.conn, include_disabled).map_err(|err| err.to_string())
    }

    fn get(&self, id: &str) -> Result<Option<Channel>, String> {
        get(self.conn, id).map_err(|err| err.to_string())
    }

    fn update(&self, id: &str, patch: &ChannelPatch) -> Result<Option<Channel>, String> {
        update(self.conn, id, patch).map_err(|err| err.to_string())
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        delete(self.conn, id).map_err(|err| err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::params;

    use crate::{
        db::migrate,
        models::{ChannelPatch, NewChannel},
    };

    use super::{delete, insert, list};

    fn setup_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn
    }

    fn new_channel(id: &str, name: &str, created_at: i64) -> NewChannel {
        NewChannel {
            id: id.to_string(),
            name: name.to_string(),
            channel_type: "openai_compatible".to_string(),
            base_url: "https://api.openai.com".to_string(),
            api_key: Some("sk-test".to_string()),
            auth_type: None,
            models_endpoint: None,
            chat_endpoint: None,
            stream_endpoint: None,
            enabled: true,
            created_at,
            updated_at: created_at,
        }
    }

    #[test]
    fn list_orders_by_created_at_desc() {
        let conn = setup_conn();
        insert(&conn, &new_channel("c1", "Old", 100)).unwrap();
        insert(&conn, &new_channel("c2", "New", 200)).unwrap();

        let channels = list(&conn, true).unwrap();

        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0].id, "c2");
        assert_eq!(channels[1].id, "c1");
    }

    #[test]
    fn delete_cascades_models_and_nulls_conversation_refs() {
        let conn = setup_conn();
        insert(&conn, &new_channel("c1", "Main", 100)).unwrap();
        conn.execute(
            "INSERT INTO api_channel_models (id, channel_id, model_id) VALUES (?1, ?2, ?3)",
            params!["m1", "c1", "gpt-4o"],
        )
        .unwrap();
        conn.execute(
            r#"
            INSERT INTO conversations (id, title, channel_id, channel_model_id, archived, pinned, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, 0, 0, ?5, ?5)
            "#,
            params!["conv1", "Test", "c1", "m1", 100],
        )
        .unwrap();

        let deleted = delete(&conn, "c1").unwrap();

        assert!(deleted);
        let model_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM api_channel_models WHERE channel_id = 'c1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(model_count, 0);

        let (channel_id, channel_model_id): (Option<String>, Option<String>) = conn
            .query_row(
                "SELECT channel_id, channel_model_id FROM conversations WHERE id = 'conv1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(channel_id, None);
        assert_eq!(channel_model_id, None);
    }

    #[test]
    fn update_changes_only_supplied_fields() {
        let conn = setup_conn();
        insert(&conn, &new_channel("c1", "Main", 100)).unwrap();

        let updated = super::update(
            &conn,
            "c1",
            &ChannelPatch {
                name: Some("Renamed".to_string()),
                base_url: None,
                channel_type: None,
                api_key: None,
                auth_type: None,
                models_endpoint: None,
                chat_endpoint: None,
                stream_endpoint: None,
                enabled: Some(false),
                updated_at: 200,
            },
        )
        .unwrap()
        .unwrap();

        assert_eq!(updated.name, "Renamed");
        assert_eq!(updated.base_url, "https://api.openai.com");
        assert!(!updated.enabled);
        assert_eq!(updated.updated_at, 200);
    }
}
