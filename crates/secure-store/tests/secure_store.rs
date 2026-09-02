use secure_store::{
    MemorySecretStore, ParentAuthorization, ParentAuthorizationResult, PinAttemptLimiter, SecretStore,
};

#[test]
fn parent_pin_is_verified_without_plaintext_round_trip() {
    let store = MemorySecretStore::default();
    store.put_secret("parent-pin", "2468").unwrap();

    assert!(store.verify_secret("parent-pin", "2468").unwrap());
    assert!(!store.verify_secret("parent-pin", "1357").unwrap());
}

#[test]
fn deleting_parent_pin_removes_authorization() {
    let store = MemorySecretStore::default();
    store.put_secret("parent-pin", "2468").unwrap();
    store.delete_secret("parent-pin").unwrap();

    assert!(!store.verify_secret("parent-pin", "2468").unwrap());
}

#[test]
fn replacing_parent_pin_invalidates_the_old_pin() {
    let store = MemorySecretStore::default();
    store.put_secret("parent-pin", "2468").unwrap();
    store.put_secret("parent-pin", "8642").unwrap();

    assert!(!store.verify_secret("parent-pin", "2468").unwrap());
    assert!(store.verify_secret("parent-pin", "8642").unwrap());
}

#[test]
fn five_failed_attempts_lock_parent_authorization_for_sixty_seconds() {
    let mut limiter = PinAttemptLimiter::default();

    for _ in 0..5 {
        assert!(limiter.record_failure(1_000).is_ok());
    }

    assert!(limiter.is_locked(1_059));
    assert!(!limiter.is_locked(1_060));
}

#[test]
fn successful_authorization_resets_failed_attempts() {
    let mut limiter = PinAttemptLimiter::default();
    for _ in 0..4 {
        limiter.record_failure(2_000).unwrap();
    }

    limiter.record_success();
    limiter.record_failure(2_001).unwrap();

    assert!(!limiter.is_locked(2_001));
}

#[test]
fn authorization_service_locks_after_five_wrong_pins() {
    let store = MemorySecretStore::default();
    store.put_secret("parent-pin", "2468").unwrap();
    let mut authorization = ParentAuthorization::new(store, "parent-pin");

    for attempt in 0..4 {
        assert_eq!(
            authorization.verify("1357", 3_000 + attempt).unwrap(),
            ParentAuthorizationResult::Denied
        );
    }

    assert_eq!(
        authorization.verify("1357", 3_004).unwrap(),
        ParentAuthorizationResult::Locked
    );
    assert_eq!(
        authorization.verify("2468", 3_059).unwrap(),
        ParentAuthorizationResult::Locked
    );
    assert_eq!(
        authorization.verify("2468", 3_064).unwrap(),
        ParentAuthorizationResult::Authorized
    );
}
