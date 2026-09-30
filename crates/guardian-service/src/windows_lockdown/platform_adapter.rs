use crate::platform::{
    PlatformAdapterError, PlatformInspection, PlatformKind, PlatformLockdownAdapter,
    PlatformLockdownConfig,
};

use super::{
    build_validated_assigned_access_config, AssignedAccessConfig, LockdownAdapterError,
    LockdownConfigError, LockdownInspection, LockdownProfile, WindowsAssignedAccessAdapter,
    WindowsLockdownAdapter,
};

fn map_error(error: LockdownAdapterError) -> PlatformAdapterError {
    match error {
        LockdownAdapterError::AccessDenied => PlatformAdapterError::AccessDenied,
        LockdownAdapterError::UnsupportedPlatform => PlatformAdapterError::UnsupportedPlatform,
        LockdownAdapterError::PlatformFailure(message) => {
            PlatformAdapterError::PlatformFailure(message)
        }
    }
}

pub fn build_windows_platform_config(
    profile: &LockdownProfile,
) -> Result<PlatformLockdownConfig, LockdownConfigError> {
    let config = build_validated_assigned_access_config(profile)?;
    PlatformLockdownConfig::new(PlatformKind::Windows, config.as_xml().to_string())
        .map_err(|_| LockdownConfigError::KidOSRequired)
}

#[derive(Debug)]
pub struct WindowsPlatformAdapter<A: WindowsLockdownAdapter> {
    inner: A,
}

impl<A: WindowsLockdownAdapter> WindowsPlatformAdapter<A> {
    pub fn new(inner: A) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &A {
        &self.inner
    }
}

impl<A> Default for WindowsPlatformAdapter<A>
where
    A: WindowsLockdownAdapter + Default,
{
    fn default() -> Self {
        Self::new(A::default())
    }
}

impl<A: WindowsLockdownAdapter> PlatformLockdownAdapter for WindowsPlatformAdapter<A> {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Windows
    }

    fn inspect(&self) -> Result<PlatformInspection, PlatformAdapterError> {
        match self.inner.inspect().map_err(map_error)? {
            LockdownInspection::NotConfigured => Ok(PlatformInspection::NotConfigured),
            LockdownInspection::Configured => Ok(PlatformInspection::Configured),
            LockdownInspection::Unsupported => Ok(PlatformInspection::Unsupported),
        }
    }

    fn apply(&mut self, config: &PlatformLockdownConfig) -> Result<(), PlatformAdapterError> {
        if config.platform() != PlatformKind::Windows {
            return Err(PlatformAdapterError::PlatformMismatch {
                expected: PlatformKind::Windows,
                received: config.platform(),
            });
        }

        let assigned_access = AssignedAccessConfig::validated(config.payload().to_string());
        self.inner.apply(&assigned_access).map_err(map_error)
    }

    fn remove(&mut self) -> Result<(), PlatformAdapterError> {
        self.inner.remove().map_err(map_error)
    }
}

pub type ProductionWindowsPlatformAdapter =
    WindowsPlatformAdapter<WindowsAssignedAccessAdapter>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::windows_lockdown::{AccountRole, ApprovedApp, InMemoryWindowsLockdownAdapter};

    fn valid_profile() -> LockdownProfile {
        LockdownProfile {
            profile_id: "{TEST}".into(),
            account: "Kid".into(),
            account_role: AccountRole::Standard,
            apps: vec![ApprovedApp {
                id: "kidos".into(),
                display_name: "KidOS".into(),
                executable_path: r"C:\Program Files\KidOS\KidOS.exe".into(),
            }],
        }
    }

    #[test]
    fn windows_profile_becomes_platform_configuration() {
        let config = build_windows_platform_config(&valid_profile()).unwrap();
        assert_eq!(config.platform(), PlatformKind::Windows);
        assert!(config.payload().contains("AssignedAccessConfiguration"));
    }

    #[test]
    fn generic_service_can_drive_windows_adapter() {
        let adapter = WindowsPlatformAdapter::new(InMemoryWindowsLockdownAdapter::default());
        let mut service = crate::platform::PlatformLockdownService::new(adapter);
        let config = build_windows_platform_config(&valid_profile()).unwrap();

        service.prepare_and_apply(&config).unwrap();
        assert_eq!(
            service.status(0).state,
            crate::platform::PlatformLockdownState::Locked
        );
    }
}
