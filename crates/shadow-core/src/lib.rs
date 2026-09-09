pub mod differential;
pub mod event;
pub mod release_gate;
pub mod result;
pub mod score;

pub use differential::compare_decisions;
pub use event::{ShadowEvent, ShadowEventType};
pub use release_gate::{evaluate_release, ReleaseDecision};
pub use result::{ShadowResult, ShadowSeverity};
pub use score::{calculate_score, ShadowScore};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_decisions_pass() {
        let result = compare_decisions("web_policy", Some("allow"), Some("allow"));
        assert!(result.passed);
        assert_eq!(result.severity, ShadowSeverity::Info);
    }

    #[test]
    fn mismatched_decisions_warn() {
        let result = compare_decisions("web_policy", Some("allow"), Some("block"));
        assert!(!result.passed);
        assert_eq!(result.severity, ShadowSeverity::Warning);
    }

    #[test]
    fn release_gate_blocks_critical_failures() {
        let score = ShadowScore {
            total: 100,
            passed: 100,
            warnings: 0,
            critical: 1,
            percentage: 100.0,
        };
        assert_eq!(evaluate_release(&score), ReleaseDecision::Blocked);
    }

    #[test]
    fn release_gate_requires_98_percent() {
        let score = ShadowScore {
            total: 100,
            passed: 97,
            warnings: 3,
            critical: 0,
            percentage: 97.0,
        };
        assert_eq!(evaluate_release(&score), ReleaseDecision::Blocked);
    }

    #[test]
    fn release_gate_approves_strong_score() {
        let score = ShadowScore {
            total: 100,
            passed: 99,
            warnings: 1,
            critical: 0,
            percentage: 99.0,
        };
        assert_eq!(evaluate_release(&score), ReleaseDecision::Approved);
    }
}
