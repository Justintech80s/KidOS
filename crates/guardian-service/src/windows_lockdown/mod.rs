mod adapter;
mod config;
mod service;
mod platform_adapter;

pub use adapter::{
    AssignedAccessConfig, InMemoryWindowsLockdownAdapter, LockdownAdapterError,
    LockdownInspection, WindowsAssignedAccessAdapter, WindowsLockdownAdapter,
};
pub use config::{
    build_assigned_access_config, AccountRole, ApprovedApp, LockdownConfigError, LockdownProfile,
};
pub use service::{
    LockdownServiceError, LockdownState, LockdownStatus, ParentUnlockGrant,
    WindowsLockdownService,
};

pub fn build_validated_assigned_access_config(
    profile: &LockdownProfile,
) -> Result<AssignedAccessConfig, LockdownConfigError> {
    build_assigned_access_config(profile).map(AssignedAccessConfig::validated)
}

pub use platform_adapter::{
    build_windows_platform_config, ProductionWindowsPlatformAdapter, WindowsPlatformAdapter,
};
