use secure_store::{MemorySecretStore, SecretStore};

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
