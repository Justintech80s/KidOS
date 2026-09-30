#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemMode {
    Healthy,
    RestrictedSafeMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreHealth {
    pub guardian_healthy: bool,
    pub policy_integrity_valid: bool,
    pub child_session_contained: bool,
}

impl CoreHealth {
    pub const fn new(
        guardian_healthy: bool,
        policy_integrity_valid: bool,
        child_session_contained: bool,
    ) -> Self {
        Self {
            guardian_healthy,
            policy_integrity_valid,
            child_session_contained,
        }
    }

    pub const fn mode(self) -> SystemMode {
        if self.guardian_healthy && self.policy_integrity_valid && self.child_session_contained {
            SystemMode::Healthy
        } else {
            SystemMode::RestrictedSafeMode
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryState {
    Healthy,
    Restricted,
    Recovering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryEvent {
    ProtectionFault,
    PolicyInvalid,
    ContainmentLost,
    RecoveryStarted,
    RecoveryVerified,
    RecoveryFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryMachine {
    state: RecoveryState,
}

impl RecoveryMachine {
    pub const fn new(initial: RecoveryState) -> Self {
        Self { state: initial }
    }

    pub const fn state(self) -> RecoveryState {
        self.state
    }

    pub fn apply(&mut self, event: RecoveryEvent) -> RecoveryState {
        self.state = match (self.state, event) {
            (_, RecoveryEvent::ProtectionFault | RecoveryEvent::PolicyInvalid | RecoveryEvent::ContainmentLost) => {
                RecoveryState::Restricted
            }
            (RecoveryState::Restricted, RecoveryEvent::RecoveryStarted) => RecoveryState::Recovering,
            (RecoveryState::Recovering, RecoveryEvent::RecoveryVerified) => RecoveryState::Healthy,
            (RecoveryState::Recovering, RecoveryEvent::RecoveryFailed) => RecoveryState::Restricted,
            (state, _) => state,
        };
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_health_is_fail_closed() {
        assert_eq!(CoreHealth::new(true, true, true).mode(), SystemMode::Healthy);
        assert_eq!(CoreHealth::new(false, true, true).mode(), SystemMode::RestrictedSafeMode);
        assert_eq!(CoreHealth::new(true, false, true).mode(), SystemMode::RestrictedSafeMode);
        assert_eq!(CoreHealth::new(true, true, false).mode(), SystemMode::RestrictedSafeMode);
    }

    #[test]
    fn recovery_requires_verified_completion() {
        let mut machine = RecoveryMachine::new(RecoveryState::Healthy);
        assert_eq!(machine.apply(RecoveryEvent::ProtectionFault), RecoveryState::Restricted);
        assert_eq!(machine.apply(RecoveryEvent::RecoveryStarted), RecoveryState::Recovering);
        assert_eq!(machine.apply(RecoveryEvent::RecoveryFailed), RecoveryState::Restricted);
        assert_eq!(machine.apply(RecoveryEvent::RecoveryStarted), RecoveryState::Recovering);
        assert_eq!(machine.apply(RecoveryEvent::RecoveryVerified), RecoveryState::Healthy);
    }
}
