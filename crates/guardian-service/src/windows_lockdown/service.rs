use super::{
    build_assigned_access_config, AssignedAccessConfig, LockdownAdapterError, LockdownInspection,
    LockdownProfile, WindowsLockdownAdapter,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockdownState {
    Unmanaged,
    Preparing,
    Locked,
    ParentUnlocked,
    RestrictedSafeMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParentUnlockGrant {
    pub expires_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockdownStatus {
    pub state: LockdownState,
    pub parent_unlock_expires_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockdownServiceError {
    Unauthorized,
    InvalidDuration,
    Configuration,
    Adapter(LockdownAdapterError),
}

pub struct WindowsLockdownService<A: WindowsLockdownAdapter> {
    adapter: A,
    state: LockdownState,
    unlock_expires_at: Option<u64>,
    last_known_valid: Option<AssignedAccessConfig>,
}

impl<A: WindowsLockdownAdapter> WindowsLockdownService<A> {
    pub fn new(adapter: A) -> Self {
        Self {
            adapter,
            state: LockdownState::Unmanaged,
            unlock_expires_at: None,
            last_known_valid: None,
        }
    }

    pub fn status(&mut self, now: u64) -> LockdownStatus {
        if let Some(expires_at) = self.unlock_expires_at {
            if now >= expires_at {
                self.unlock_expires_at = None;
                if self.state == LockdownState::ParentUnlocked {
                    self.state = LockdownState::Locked;
                }
            }
        }

        if matches!(self.state, LockdownState::Locked | LockdownState::ParentUnlocked) {
            match self.adapter.inspect() {
                Ok(LockdownInspection::Configured) => {}
                Ok(LockdownInspection::NotConfigured | LockdownInspection::Unsupported) | Err(_) => {
                    self.state = LockdownState::RestrictedSafeMode;
                    self.unlock_expires_at = None;
                }
            }
        }

        LockdownStatus {
            state: self.state,
            parent_unlock_expires_at: self.unlock_expires_at,
        }
    }

    pub fn prepare_and_apply(&mut self, profile: &LockdownProfile) -> Result<(), LockdownServiceError> {
        self.state = LockdownState::Preparing;
        let xml = build_assigned_access_config(profile).map_err(|_| {
            self.state = LockdownState::RestrictedSafeMode;
            LockdownServiceError::Configuration
        })?;
        let config = AssignedAccessConfig::validated(xml);
        if let Err(error) = self.adapter.apply(&config) {
            self.state = LockdownState::RestrictedSafeMode;
            return Err(LockdownServiceError::Adapter(error));
        }
        self.last_known_valid = Some(config);
        self.state = LockdownState::Locked;
        Ok(())
    }

    pub fn begin_parent_unlock(
        &mut self,
        parent_authorized: bool,
        now: u64,
        duration_minutes: u64,
    ) -> Result<ParentUnlockGrant, LockdownServiceError> {
        if !parent_authorized {
            return Err(LockdownServiceError::Unauthorized);
        }
        if duration_minutes == 0 || duration_minutes > 60 {
            return Err(LockdownServiceError::InvalidDuration);
        }
        if self.state != LockdownState::Locked {
            return Err(LockdownServiceError::Configuration);
        }
        let expires_at = now.saturating_add(duration_minutes.saturating_mul(60));
        self.unlock_expires_at = Some(expires_at);
        self.state = LockdownState::ParentUnlocked;
        Ok(ParentUnlockGrant { expires_at })
    }

    pub fn remove_lockdown(&mut self, parent_authorized: bool) -> Result<(), LockdownServiceError> {
        if !parent_authorized {
            return Err(LockdownServiceError::Unauthorized);
        }
        self.adapter.remove().map_err(LockdownServiceError::Adapter)?;
        self.state = LockdownState::Unmanaged;
        self.unlock_expires_at = None;
        self.last_known_valid = None;
        Ok(())
    }

    pub fn last_known_valid_config(&self) -> Option<&AssignedAccessConfig> {
        self.last_known_valid.as_ref()
    }
}
