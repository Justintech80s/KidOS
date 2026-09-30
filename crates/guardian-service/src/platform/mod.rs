use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformKind {
    Windows,
    Linux,
    Android,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformLockdownConfig {
    platform: PlatformKind,
    payload: String,
}

impl PlatformLockdownConfig {
    pub fn new(platform: PlatformKind, payload: impl Into<String>) -> Result<Self, PlatformAdapterError> {
        let payload = payload.into();
        if payload.trim().is_empty() {
            return Err(PlatformAdapterError::InvalidConfiguration(
                "platform lockdown payload must not be empty".into(),
            ));
        }
        Ok(Self { platform, payload })
    }

    pub const fn platform(&self) -> PlatformKind {
        self.platform
    }

    pub fn payload(&self) -> &str {
        &self.payload
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformInspection {
    NotConfigured,
    Configured,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformAdapterError {
    AccessDenied,
    UnsupportedPlatform,
    PlatformMismatch {
        expected: PlatformKind,
        received: PlatformKind,
    },
    InvalidConfiguration(String),
    PlatformFailure(String),
}

impl fmt::Display for PlatformAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AccessDenied => write!(formatter, "platform adapter access denied"),
            Self::UnsupportedPlatform => write!(formatter, "platform adapter is unsupported"),
            Self::PlatformMismatch { expected, received } => write!(
                formatter,
                "platform configuration mismatch: expected {:?}, received {:?}",
                expected, received
            ),
            Self::InvalidConfiguration(message) => write!(formatter, "invalid platform configuration: {message}"),
            Self::PlatformFailure(message) => write!(formatter, "platform adapter failure: {message}"),
        }
    }
}

impl Error for PlatformAdapterError {}

pub trait PlatformLockdownAdapter {
    fn kind(&self) -> PlatformKind;
    fn inspect(&self) -> Result<PlatformInspection, PlatformAdapterError>;
    fn apply(&mut self, config: &PlatformLockdownConfig) -> Result<(), PlatformAdapterError>;
    fn remove(&mut self) -> Result<(), PlatformAdapterError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformLockdownState {
    Unmanaged,
    Preparing,
    Locked,
    ParentUnlocked,
    RestrictedSafeMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformUnlockGrant {
    pub expires_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformLockdownStatus {
    pub state: PlatformLockdownState,
    pub parent_unlock_expires_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformLockdownServiceError {
    Unauthorized,
    InvalidDuration,
    InvalidState,
    Adapter(PlatformAdapterError),
}

pub struct PlatformLockdownService<A: PlatformLockdownAdapter> {
    adapter: A,
    state: PlatformLockdownState,
    unlock_expires_at: Option<u64>,
    last_known_valid: Option<PlatformLockdownConfig>,
}

impl<A: PlatformLockdownAdapter> PlatformLockdownService<A> {
    pub fn new(adapter: A) -> Self {
        Self {
            adapter,
            state: PlatformLockdownState::Unmanaged,
            unlock_expires_at: None,
            last_known_valid: None,
        }
    }

    pub fn adapter(&self) -> &A {
        &self.adapter
    }

    pub fn status(&mut self, now: u64) -> PlatformLockdownStatus {
        if let Some(expires_at) = self.unlock_expires_at {
            if now >= expires_at {
                self.unlock_expires_at = None;
                if self.state == PlatformLockdownState::ParentUnlocked {
                    self.state = PlatformLockdownState::Locked;
                }
            }
        }

        if matches!(
            self.state,
            PlatformLockdownState::Locked | PlatformLockdownState::ParentUnlocked
        ) {
            match self.adapter.inspect() {
                Ok(PlatformInspection::Configured) => {}
                Ok(PlatformInspection::NotConfigured | PlatformInspection::Unsupported) | Err(_) => {
                    self.state = PlatformLockdownState::RestrictedSafeMode;
                    self.unlock_expires_at = None;
                }
            }
        }

        PlatformLockdownStatus {
            state: self.state,
            parent_unlock_expires_at: self.unlock_expires_at,
        }
    }

    pub fn prepare_and_apply(
        &mut self,
        config: &PlatformLockdownConfig,
    ) -> Result<(), PlatformLockdownServiceError> {
        self.state = PlatformLockdownState::Preparing;

        if config.platform() != self.adapter.kind() {
            self.state = PlatformLockdownState::RestrictedSafeMode;
            return Err(PlatformLockdownServiceError::Adapter(
                PlatformAdapterError::PlatformMismatch {
                    expected: self.adapter.kind(),
                    received: config.platform(),
                },
            ));
        }

        if let Err(error) = self.adapter.apply(config) {
            self.state = PlatformLockdownState::RestrictedSafeMode;
            return Err(PlatformLockdownServiceError::Adapter(error));
        }

        self.last_known_valid = Some(config.clone());
        self.state = PlatformLockdownState::Locked;
        Ok(())
    }

    pub fn begin_parent_unlock(
        &mut self,
        parent_authorized: bool,
        now: u64,
        duration_minutes: u64,
    ) -> Result<PlatformUnlockGrant, PlatformLockdownServiceError> {
        if !parent_authorized {
            return Err(PlatformLockdownServiceError::Unauthorized);
        }
        if duration_minutes == 0 || duration_minutes > 60 {
            return Err(PlatformLockdownServiceError::InvalidDuration);
        }
        if self.state != PlatformLockdownState::Locked {
            return Err(PlatformLockdownServiceError::InvalidState);
        }

        let expires_at = now.saturating_add(duration_minutes.saturating_mul(60));
        self.unlock_expires_at = Some(expires_at);
        self.state = PlatformLockdownState::ParentUnlocked;
        Ok(PlatformUnlockGrant { expires_at })
    }

    pub fn remove_lockdown(
        &mut self,
        parent_authorized: bool,
    ) -> Result<(), PlatformLockdownServiceError> {
        if !parent_authorized {
            return Err(PlatformLockdownServiceError::Unauthorized);
        }

        self.adapter
            .remove()
            .map_err(PlatformLockdownServiceError::Adapter)?;
        self.state = PlatformLockdownState::Unmanaged;
        self.unlock_expires_at = None;
        self.last_known_valid = None;
        Ok(())
    }

    pub fn last_known_valid_config(&self) -> Option<&PlatformLockdownConfig> {
        self.last_known_valid.as_ref()
    }
}

#[derive(Debug)]
pub struct InMemoryPlatformAdapter {
    kind: PlatformKind,
    configured: bool,
    failure: Option<PlatformAdapterError>,
}

impl InMemoryPlatformAdapter {
    pub fn new(kind: PlatformKind) -> Self {
        Self {
            kind,
            configured: false,
            failure: None,
        }
    }

    pub fn with_failure(kind: PlatformKind, error: PlatformAdapterError) -> Self {
        Self {
            kind,
            configured: false,
            failure: Some(error),
        }
    }

    fn fail_if_requested(&self) -> Result<(), PlatformAdapterError> {
        match &self.failure {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }
}

impl PlatformLockdownAdapter for InMemoryPlatformAdapter {
    fn kind(&self) -> PlatformKind {
        self.kind
    }

    fn inspect(&self) -> Result<PlatformInspection, PlatformAdapterError> {
        self.fail_if_requested()?;
        Ok(if self.configured {
            PlatformInspection::Configured
        } else {
            PlatformInspection::NotConfigured
        })
    }

    fn apply(&mut self, config: &PlatformLockdownConfig) -> Result<(), PlatformAdapterError> {
        self.fail_if_requested()?;
        if config.platform() != self.kind {
            return Err(PlatformAdapterError::PlatformMismatch {
                expected: self.kind,
                received: config.platform(),
            });
        }
        self.configured = true;
        Ok(())
    }

    fn remove(&mut self) -> Result<(), PlatformAdapterError> {
        self.fail_if_requested()?;
        self.configured = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(kind: PlatformKind) -> PlatformLockdownConfig {
        PlatformLockdownConfig::new(kind, "test-config").unwrap()
    }

    #[test]
    fn applies_matching_platform_configuration() {
        let adapter = InMemoryPlatformAdapter::new(PlatformKind::Linux);
        let mut service = PlatformLockdownService::new(adapter);
        service.prepare_and_apply(&config(PlatformKind::Linux)).unwrap();
        assert_eq!(
            service.status(0).state,
            PlatformLockdownState::Locked
        );
    }

    #[test]
    fn rejects_configuration_for_another_platform() {
        let adapter = InMemoryPlatformAdapter::new(PlatformKind::Linux);
        let mut service = PlatformLockdownService::new(adapter);
        assert!(service
            .prepare_and_apply(&config(PlatformKind::Android))
            .is_err());
        assert_eq!(
            service.status(0).state,
            PlatformLockdownState::RestrictedSafeMode
        );
    }

    #[test]
    fn adapter_failure_forces_restricted_safe_mode() {
        let adapter = InMemoryPlatformAdapter::with_failure(
            PlatformKind::Windows,
            PlatformAdapterError::PlatformFailure("injected".into()),
        );
        let mut service = PlatformLockdownService::new(adapter);
        assert!(service
            .prepare_and_apply(&config(PlatformKind::Windows))
            .is_err());
        assert_eq!(
            service.status(0).state,
            PlatformLockdownState::RestrictedSafeMode
        );
    }

    #[test]
    fn parent_unlock_expires_back_to_locked() {
        let adapter = InMemoryPlatformAdapter::new(PlatformKind::Android);
        let mut service = PlatformLockdownService::new(adapter);
        service
            .prepare_and_apply(&config(PlatformKind::Android))
            .unwrap();
        let grant = service.begin_parent_unlock(true, 100, 5).unwrap();
        assert_eq!(grant.expires_at, 400);
        assert_eq!(
            service.status(399).state,
            PlatformLockdownState::ParentUnlocked
        );
        assert_eq!(service.status(400).state, PlatformLockdownState::Locked);
    }
}
