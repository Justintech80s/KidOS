use guardian_service::windows_lockdown::{
    LockdownAdapterError, WindowsAssignedAccessAdapter, WindowsLockdownAdapter,
};

#[cfg(target_os = "windows")]
#[test]
fn production_adapter_refuses_non_system_processes_before_touching_mdm_bridge() {
    let adapter = WindowsAssignedAccessAdapter::default();
    assert_eq!(adapter.inspect(), Err(LockdownAdapterError::AccessDenied));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn production_adapter_remains_unsupported_off_windows() {
    let adapter = WindowsAssignedAccessAdapter::default();
    assert!(matches!(
        adapter.inspect(),
        Ok(guardian_service::windows_lockdown::LockdownInspection::Unsupported)
    ));
}
