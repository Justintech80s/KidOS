use crate::PolicySnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardianMode {
    Healthy,
    RestrictedSafeMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardianState {
    pub mode: GuardianMode,
    pub policy: PolicySnapshot,
}

pub fn load_service_state(
    current: Option<PolicySnapshot>,
    last_known_valid: Option<PolicySnapshot>,
) -> GuardianState {
    if let Some(policy) = current.filter(|policy| policy.integrity_valid) {
        return GuardianState {
            mode: GuardianMode::Healthy,
            policy: policy.with_source("current"),
        };
    }

    if let Some(policy) = last_known_valid.filter(|policy| policy.integrity_valid) {
        return GuardianState {
            mode: GuardianMode::Healthy,
            policy: policy.with_source("last_known_valid"),
        };
    }

    GuardianState {
        mode: GuardianMode::RestrictedSafeMode,
        policy: PolicySnapshot::strict_baseline(),
    }
}
