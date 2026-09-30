use crate::GuardianMode;
use os_foundation::{CoreHealth, SystemMode};

pub fn guardian_mode_from_core_health(
    guardian_healthy: bool,
    policy_integrity_valid: bool,
    child_session_contained: bool,
) -> GuardianMode {
    match CoreHealth::new(
        guardian_healthy,
        policy_integrity_valid,
        child_session_contained,
    )
    .mode()
    {
        SystemMode::Healthy => GuardianMode::Healthy,
        SystemMode::RestrictedSafeMode => GuardianMode::RestrictedSafeMode,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_health_fails_closed_for_every_core_failure() {
        assert_eq!(
            guardian_mode_from_core_health(true, true, true),
            GuardianMode::Healthy
        );
        assert_eq!(
            guardian_mode_from_core_health(false, true, true),
            GuardianMode::RestrictedSafeMode
        );
        assert_eq!(
            guardian_mode_from_core_health(true, false, true),
            GuardianMode::RestrictedSafeMode
        );
        assert_eq!(
            guardian_mode_from_core_health(true, true, false),
            GuardianMode::RestrictedSafeMode
        );
    }
}
