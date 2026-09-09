use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShadowSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowResult {
    pub test_name: String,
    pub passed: bool,
    pub severity: ShadowSeverity,
    pub production_decision: Option<String>,
    pub shadow_decision: Option<String>,
    pub latency_ms: Option<u128>,
    pub reason: String,
}
