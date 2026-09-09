use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShadowEventType {
    WebNavigation,
    Download,
    AppLaunch,
    MediaScan,
    PolicyDecision,
    GuardianHealth,
    ClassifierHealth,
    Recovery,
    Update,
    Authentication,
    SystemStartup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowEvent {
    pub id: String,
    pub event_type: ShadowEventType,
    pub timestamp_ms: u128,
    pub subject: String,
    pub production_decision: Option<String>,
    pub metadata: serde_json::Value,
}

impl ShadowEvent {
    pub fn new(
        id: impl Into<String>,
        event_type: ShadowEventType,
        subject: impl Into<String>,
        production_decision: Option<String>,
        metadata: serde_json::Value,
    ) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        Self {
            id: id.into(),
            event_type,
            timestamp_ms,
            subject: subject.into(),
            production_decision,
            metadata,
        }
    }
}
