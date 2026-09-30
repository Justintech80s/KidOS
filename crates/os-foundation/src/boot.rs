use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BootPhase {
    Firmware,
    Kernel,
    CoreServices,
    GuardianVerified,
    ChildSession,
    Ready,
}

impl BootPhase {
    const fn next(self) -> Option<Self> {
        match self {
            Self::Firmware => Some(Self::Kernel),
            Self::Kernel => Some(Self::CoreServices),
            Self::CoreServices => Some(Self::GuardianVerified),
            Self::GuardianVerified => Some(Self::ChildSession),
            Self::ChildSession => Some(Self::Ready),
            Self::Ready => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootSequence {
    phase: BootPhase,
}

impl Default for BootSequence {
    fn default() -> Self {
        Self { phase: BootPhase::Firmware }
    }
}

impl BootSequence {
    pub const fn phase(self) -> BootPhase {
        self.phase
    }

    pub fn advance(&mut self, target: BootPhase) -> Result<(), BootError> {
        if self.phase.next() == Some(target) {
            self.phase = target;
            Ok(())
        } else {
            Err(BootError {
                current: self.phase,
                requested: target,
            })
        }
    }

    pub const fn child_session_may_start(self) -> bool {
        matches!(
            self.phase,
            BootPhase::GuardianVerified | BootPhase::ChildSession | BootPhase::Ready
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootError {
    pub current: BootPhase,
    pub requested: BootPhase,
}

impl fmt::Display for BootError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid boot transition from {:?} to {:?}",
            self.current, self.requested
        )
    }
}

impl Error for BootError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_sequence_cannot_skip_guardian_verification() {
        let mut boot = BootSequence::default();
        boot.advance(BootPhase::Kernel).unwrap();
        boot.advance(BootPhase::CoreServices).unwrap();
        assert!(boot.advance(BootPhase::ChildSession).is_err());
        assert!(!boot.child_session_may_start());
        boot.advance(BootPhase::GuardianVerified).unwrap();
        assert!(boot.child_session_may_start());
    }
}
