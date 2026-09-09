use crate::result::{ShadowResult, ShadowSeverity};

pub fn compare_decisions(
    test_name: &str,
    production: Option<&str>,
    shadow: Option<&str>,
) -> ShadowResult {
    let passed = production == shadow;

    ShadowResult {
        test_name: test_name.to_string(),
        passed,
        severity: if passed {
            ShadowSeverity::Info
        } else {
            ShadowSeverity::Warning
        },
        production_decision: production.map(str::to_owned),
        shadow_decision: shadow.map(str::to_owned),
        latency_ms: None,
        reason: if passed {
            "Production and Shadow decisions match.".into()
        } else {
            format!(
                "Decision mismatch: production={:?}, shadow={:?}",
                production, shadow
            )
        },
    }
}
