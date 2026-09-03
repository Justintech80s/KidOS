use guardian_service::windows_lockdown::{
    AssignedAccessConfig, InMemoryWindowsLockdownAdapter, LockdownAdapterError,
    LockdownInspection, WindowsLockdownAdapter,
};

#[test]
fn adapter_applies_only_validated_configuration_objects() {
    let mut adapter = InMemoryWindowsLockdownAdapter::default();
    let config = AssignedAccessConfig::validated("<AssignedAccessConfiguration />".into());

    adapter.apply(&config).expect("validated config should apply");
    assert_eq!(adapter.inspect().unwrap(), LockdownInspection::Configured);
}

#[test]
fn remove_has_no_arbitrary_command_parameter() {
    let mut adapter = InMemoryWindowsLockdownAdapter::default();
    let config = AssignedAccessConfig::validated("<AssignedAccessConfiguration />".into());
    adapter.apply(&config).unwrap();

    adapter.remove().unwrap();
    assert_eq!(adapter.inspect().unwrap(), LockdownInspection::NotConfigured);
}

#[test]
fn failures_are_typed() {
    let mut adapter = InMemoryWindowsLockdownAdapter::with_failure(LockdownAdapterError::AccessDenied);
    let config = AssignedAccessConfig::validated("<AssignedAccessConfiguration />".into());

    assert_eq!(adapter.apply(&config), Err(LockdownAdapterError::AccessDenied));
}
