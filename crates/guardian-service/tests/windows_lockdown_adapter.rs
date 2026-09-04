use guardian_service::windows_lockdown::{
    build_validated_assigned_access_config, AccountRole, ApprovedApp,
    InMemoryWindowsLockdownAdapter, LockdownAdapterError, LockdownInspection, LockdownProfile,
    WindowsLockdownAdapter,
};

fn profile() -> LockdownProfile {
    LockdownProfile {
        profile_id: "11111111-1111-1111-1111-111111111111".into(),
        account: "KidOSChild".into(),
        account_role: AccountRole::Standard,
        apps: vec![ApprovedApp {
            id: "kidos".into(),
            display_name: "KidOS".into(),
            executable_path: r"C:\Program Files\KidOS\KidOS.exe".into(),
        }],
    }
}

#[test]
fn adapter_applies_only_profile_built_configuration_objects() {
    let mut adapter = InMemoryWindowsLockdownAdapter::default();
    let config = build_validated_assigned_access_config(&profile()).unwrap();

    adapter.apply(&config).expect("profile-built config should apply");
    assert_eq!(adapter.inspect().unwrap(), LockdownInspection::Configured);
}

#[test]
fn remove_has_no_arbitrary_command_parameter() {
    let mut adapter = InMemoryWindowsLockdownAdapter::default();
    let config = build_validated_assigned_access_config(&profile()).unwrap();
    adapter.apply(&config).unwrap();

    adapter.remove().unwrap();
    assert_eq!(adapter.inspect().unwrap(), LockdownInspection::NotConfigured);
}

#[test]
fn failures_are_typed() {
    let mut adapter = InMemoryWindowsLockdownAdapter::with_failure(LockdownAdapterError::AccessDenied);
    let config = build_validated_assigned_access_config(&profile()).unwrap();

    assert_eq!(adapter.apply(&config), Err(LockdownAdapterError::AccessDenied));
}
