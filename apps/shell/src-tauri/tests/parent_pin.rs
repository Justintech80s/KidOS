use kidos_shell_lib::configure_parent_pin_with_store;
use secure_store::{MemorySecretStore, SecretStore};

#[test]
fn parent_pin_configuration_rejects_invalid_input() {
    let store = MemorySecretStore::default();
    assert!(configure_parent_pin_with_store(&store, "12ab").is_err());
    assert!(!store.verify_secret("parent-pin", "12ab").unwrap());
}

#[test]
fn parent_pin_configuration_stores_only_a_verifiable_secret() {
    let store = MemorySecretStore::default();
    configure_parent_pin_with_store(&store, "2468").unwrap();

    assert!(store.verify_secret("parent-pin", "2468").unwrap());
    assert!(!store.verify_secret("parent-pin", "1357").unwrap());
}
