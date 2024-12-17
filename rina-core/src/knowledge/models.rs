use super::types::{ChannelType, Source};
use rig_sqlite::{Column, ColumnValue, SqliteVectorStoreTable};
use rig::Embed;
use rusqlite::Row;

#[derive(Embed, Clone, Debug)]
pub struct Document {
    pub id: String,
    pub source_id: String,
    #[embed]
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub source: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Conversation {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Embed, Clone, Debug, serde::Deserialize)]
pub struct Message {
    pub id: String,
    pub source: Source,
    pub source_id: String,
    pub channel_type: ChannelType,
    pub channel_id: String,
    pub account_id: String,
    pub role: String,
    #[embed]
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub source: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// Implement the table traits
impl SqliteVectorStoreTable for Document {
    fn name() -> &'static str {
        "documents"
    }

    fn schema() -> Vec<Column> {
        vec![
            Column::new("id", "TEXT PRIMARY KEY"),
            Column::new("source_id", "TEXT").indexed(),
            Column::new("content", "TEXT"),
            Column::new("created_at", "TIMESTAMP DEFAULT CURRENT_TIMESTAMP"),
        ]
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn column_values(&self) -> Vec<(&'static str, Box<dyn ColumnValue>)> {
        vec![
            ("id", Box::new(self.id.clone())),
            ("source_id", Box::new(self.source_id.clone())),
            ("content", Box::new(self.content.clone())),
            ("created_at", Box::new(self.created_at.to_rfc3339())),
        ]
    }
}

impl SqliteVectorStoreTable for Message {
    fn name() -> &'static str {
        "messages"
    }

    fn schema() -> Vec<Column> {
        vec![
            Column::new("id", "TEXT PRIMARY KEY"),
            Column::new("source", "TEXT"),
            Column::new("source_id", "TEXT").indexed(),
            Column::new("channel_type", "TEXT"),
            Column::new("channel_id", "TEXT").indexed(),
            Column::new("account_id", "TEXT").indexed(),
            Column::new("role", "TEXT"),
            Column::new("content", "TEXT"),
            Column::new("created_at", "TIMESTAMP DEFAULT CURRENT_TIMESTAMP"),
        ]
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn column_values(&self) -> Vec<(&'static str, Box<dyn ColumnValue>)> {
        vec![
            ("id", Box::new(self.id.clone())),
            ("source", Box::new(self.source.as_str().to_string())),
            ("source_id", Box::new(self.source_id.clone())),
            (
                "channel_type",
                Box::new(self.channel_type.as_str().to_string()),
            ),
            ("channel_id", Box::new(self.channel_id.clone())),
            ("account_id", Box::new(self.account_id.clone())),
            ("role", Box::new(self.role.clone())),
            ("content", Box::new(self.content.clone())),
            ("created_at", Box::new(self.created_at.to_rfc3339())),
        ]
    }
}

impl TryFrom<&Row<'_>> for Document {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Document {
            id: row.get(0)?,
            source_id: row.get(1)?,
            content: row.get(2)?,
            created_at: row.get(3)?,
        })
    }
}

impl TryFrom<&Row<'_>> for Account {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            source: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }
}

impl TryFrom<&Row<'_>> for Conversation {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Conversation {
            id: row.get(0)?,
            user_id: row.get(1)?,
            title: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }
}

impl TryFrom<&Row<'_>> for Message {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Message {
            id: row.get(0)?,
            source: Source::from_str(&row.get::<_, String>(1)?).ok_or(
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(super::error::ConversionError("Invalid source".to_string())),
                ),
            )?,
            source_id: row.get(2)?,
            channel_type: ChannelType::from_str(&row.get::<_, String>(3)?).ok_or(
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(super::error::ConversionError(
                        "Invalid channel type".to_string(),
                    )),
                ),
            )?,
            channel_id: row.get(4)?,
            account_id: row.get(5)?,
            role: row.get(6)?,
            content: row.get(7)?,
            created_at: row.get(8)?,
        })
    }
}

impl TryFrom<&Row<'_>> for Channel {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Channel {
            id: row.get(0)?,
            name: row.get(1)?,
            source: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }
}

impl SqliteVectorStoreTable for Channel {
    fn name() -> &'static str {
        "channels"
    }

    fn schema() -> Vec<Column> {
        vec![
            Column::new("id", "TEXT PRIMARY KEY"),
            Column::new("name", "TEXT"),
            Column::new("source", "TEXT"),
            Column::new("created_at", "TIMESTAMP DEFAULT CURRENT_TIMESTAMP"),
            Column::new("updated_at", "TIMESTAMP DEFAULT CURRENT_TIMESTAMP"),
        ]
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn column_values(&self) -> Vec<(&'static str, Box<dyn ColumnValue>)> {
        vec![
            ("id", Box::new(self.id.clone())),
            ("name", Box::new(self.name.clone())),
            ("source", Box::new(self.source.clone())),
            ("created_at", Box::new(self.created_at.to_rfc3339())),
            ("updated_at", Box::new(self.updated_at.to_rfc3339())),
        ]
    }
}
