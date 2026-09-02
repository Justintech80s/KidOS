use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyEvent {
    pub timestamp: i64,
    pub action_class: String,
    pub normalized_domain: Option<String>,
    pub decision: String,
    pub reason: String,
    pub media_category: Option<String>,
    pub confidence_band: Option<String>,
    pub risk: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SafetyEventSummary {
    pub total: u64,
    pub allowed: u64,
    pub blocked: u64,
    pub parent_gated: u64,
}

#[derive(Debug)]
pub enum SafetyEventError {
    Database(rusqlite::Error),
    InvalidDomain,
    InvalidDecision,
}

impl fmt::Display for SafetyEventError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "safety event database error: {error}"),
            Self::InvalidDomain => write!(formatter, "safety event domain must contain only a normalized host name"),
            Self::InvalidDecision => write!(formatter, "safety event decision is invalid"),
        }
    }
}

impl Error for SafetyEventError {}

impl From<rusqlite::Error> for SafetyEventError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Database(value)
    }
}

pub struct SafetyEventStore {
    connection: Connection,
}

impl SafetyEventStore {
    pub fn in_memory() -> Result<Self, SafetyEventError> {
        let connection = Connection::open_in_memory()?;
        let store = Self { connection };
        store.initialize()?;
        Ok(store)
    }

    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, SafetyEventError> {
        let connection = Connection::open(path)?;
        let store = Self { connection };
        store.initialize()?;
        Ok(store)
    }

    fn initialize(&self) -> Result<(), SafetyEventError> {
        self.connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS safety_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                action_class TEXT NOT NULL,
                normalized_domain TEXT,
                decision TEXT NOT NULL,
                reason TEXT NOT NULL,
                media_category TEXT,
                confidence_band TEXT,
                risk TEXT
            );
            CREATE INDEX IF NOT EXISTS safety_events_timestamp_idx ON safety_events(timestamp DESC);",
        )?;
        Ok(())
    }

    pub fn record(&self, event: &SafetyEvent) -> Result<(), SafetyEventError> {
        if !matches!(event.decision.as_str(), "allow" | "block" | "require_parent") {
            return Err(SafetyEventError::InvalidDecision);
        }
        if let Some(domain) = &event.normalized_domain {
            if !is_normalized_domain(domain) {
                return Err(SafetyEventError::InvalidDomain);
            }
        }

        self.connection.execute(
            "INSERT INTO safety_events (
                timestamp, action_class, normalized_domain, decision, reason,
                media_category, confidence_band, risk
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                event.timestamp,
                event.action_class,
                event.normalized_domain,
                event.decision,
                event.reason,
                event.media_category,
                event.confidence_band,
                event.risk,
            ],
        )?;
        Ok(())
    }

    pub fn recent(&self, limit: u32) -> Result<Vec<SafetyEvent>, SafetyEventError> {
        let mut statement = self.connection.prepare(
            "SELECT timestamp, action_class, normalized_domain, decision, reason,
                    media_category, confidence_band, risk
             FROM safety_events
             ORDER BY timestamp DESC, id DESC
             LIMIT ?1",
        )?;
        let rows = statement.query_map([i64::from(limit)], |row| {
            Ok(SafetyEvent {
                timestamp: row.get(0)?,
                action_class: row.get(1)?,
                normalized_domain: row.get(2)?,
                decision: row.get(3)?,
                reason: row.get(4)?,
                media_category: row.get(5)?,
                confidence_band: row.get(6)?,
                risk: row.get(7)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn summary(&self) -> Result<SafetyEventSummary, SafetyEventError> {
        let summary = self.connection.query_row(
            "SELECT
                COUNT(*),
                SUM(CASE WHEN decision = 'allow' THEN 1 ELSE 0 END),
                SUM(CASE WHEN decision = 'block' THEN 1 ELSE 0 END),
                SUM(CASE WHEN decision = 'require_parent' THEN 1 ELSE 0 END)
             FROM safety_events",
            [],
            |row| {
                Ok(SafetyEventSummary {
                    total: row.get::<_, u64>(0)?,
                    allowed: row.get::<_, Option<u64>>(1)?.unwrap_or(0),
                    blocked: row.get::<_, Option<u64>>(2)?.unwrap_or(0),
                    parent_gated: row.get::<_, Option<u64>>(3)?.unwrap_or(0),
                })
            },
        )?;
        Ok(summary)
    }

    pub fn clear(&self) -> Result<(), SafetyEventError> {
        self.connection.execute("DELETE FROM safety_events", [])?;
        Ok(())
    }

    pub fn schema_columns(&self) -> Result<Vec<String>, SafetyEventError> {
        let mut statement = self.connection.prepare("PRAGMA table_info(safety_events)")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

fn is_normalized_domain(domain: &str) -> bool {
    let trimmed = domain.trim();
    !trimmed.is_empty()
        && trimmed == domain
        && trimmed.len() <= 253
        && !trimmed.contains('/')
        && !trimmed.contains('?')
        && !trimmed.contains('#')
        && !trimmed.contains(':')
        && trimmed.contains('.')
        && trimmed
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
}
