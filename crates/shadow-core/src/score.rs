use crate::result::{ShadowResult, ShadowSeverity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowScore {
    pub total: usize,
    pub passed: usize,
    pub warnings: usize,
    pub critical: usize,
    pub percentage: f64,
}

pub fn calculate_score(results: &[ShadowResult]) -> ShadowScore {
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let warnings = results
        .iter()
        .filter(|r| r.severity == ShadowSeverity::Warning)
        .count();
    let critical = results
        .iter()
        .filter(|r| r.severity == ShadowSeverity::Critical)
        .count();

    let percentage = if total == 0 {
        0.0
    } else {
        (passed as f64 / total as f64) * 100.0
    };

    ShadowScore {
        total,
        passed,
        warnings,
        critical,
        percentage,
    }
}
