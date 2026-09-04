use guardian_service::{
    AccountRole, ApprovedApp, InMemoryWindowsLockdownAdapter, LockdownAdapterError,
    LockdownProfile, LockdownServiceError, LockdownState, WindowsLockdownService,
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
fn applies_lockdown_and_reports_locked() {
    let adapter = InMemoryWindowsLockdownAdapter::default();
    let mut service = WindowsLockdownService::new(adapter);
    service.prepare_and_apply(&profile()).unwrap();
    assert_eq!(service.status(1_000).state, LockdownState::Locked);
}

#[test]
fn adapter_failure_fails_closed() {
    let adapter = InMemoryWindowsLockdownAdapter::with_failure(
        LockdownAdapterError::PlatformFailure("boom".into()),
    );
    let mut service = WindowsLockdownService::new(adapter);
    assert!(service.prepare_and_apply(&profile()).is_err());
    assert_eq!(service.status(1_000).state, LockdownState::RestrictedSafeMode);
}

#[test]
fn parent_unlock_requires_authorization_and_expires() {
    let adapter = InMemoryWindowsLockdownAdapter::default();
    let mut service = WindowsLockdownService::new(adapter);
    service.prepare_and_apply(&profile()).unwrap();

    assert_eq!(
        service.begin_parent_unlock(false, 1_000, 5),
        Err(LockdownServiceError::Unauthorized)
    );

    let grant = service.begin_parent_unlock(true, 1_000, 5).unwrap();
    assert_eq!(grant.expires_at, 1_300);
    assert_eq!(service.status(1_299).state, LockdownState::ParentUnlocked);
    assert_eq!(service.status(1_300).state, LockdownState::Locked);
}

#[test]
fn removal_requires_parent_authorization() {
    let adapter = InMemoryWindowsLockdownAdapter::default();
    let mut service = WindowsLockdownService::new(adapter);
    service.prepare_and_apply(&profile()).unwrap();
    assert_eq!(service.remove_lockdown(false), Err(LockdownServiceError::Unauthorized));
    service.remove_lockdown(true).unwrap();
    assert_eq!(service.status(1_000).state, LockdownState::Unmanaged);
}
