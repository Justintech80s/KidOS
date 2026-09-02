use secure_store::{MemorySecretStore, PinAttemptLimiter, SecretStore};

#[test]
fn parent_pin_is_verified_without_plaintext_round_trip() {
    let store = MemorySecretStore::default();
    store.put_secret("parent-pin", "4821").unwrap();

    assert!(store.verify_secret("parent-pin", "4821").unwrap());
    assert!(!store.verify_secret("parent-pin", "1111").unwrap());
}

#[test]
fn deleting_parent_pin_removes_authorization() {
    let store = MemorySecretStore::default();
    store.put_secret("parent-pin", "4821").unwrap();
    store.delete_secret("parent-pin").unwrap();

    assert!(!store.verify_secret("parent-pin", "4821").unwrap());
}

#[test]
fn replacing_parent_pin_invalidates_the_old_pin() {
    let store = MemorySecretStore::default();
    store.put_secret("parent-pin", "4821").unwrap();
    store.put_secret("parent-pin", "7394").unwrap();

    assert!(!store.verify_secret("parent-pin", "4821").unwrap());
    assert!(store.verify_secret("parent-pin", "7394").unwrap());
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
