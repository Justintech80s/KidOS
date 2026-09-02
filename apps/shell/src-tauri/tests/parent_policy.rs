use guardian_service::{
    GuardianPolicyStore, ParentDownloadMode, ParentPolicyConfig, SocialAccessMode, SocialAccessRule,
};
use kidos_shell_lib::save_parent_policy_with_authorization;
use secure_store::{MemorySecretStore, ParentAuthorization, SecretStore};

fn policy() -> ParentPolicyConfig {
    ParentPolicyConfig {
        child_age: 14,
        allow_domains: vec!["khanacademy.org".into()],
        block_domains: vec!["unsafe.example".into()],
        teen_unknown_web_enabled: true,
        social_access: vec![SocialAccessRule {
            service: "youtube".into(),
            mode: SocialAccessMode::Allowed,
            start_minutes: None,
            end_minutes: None,
        }],
        download_mode: ParentDownloadMode::BlockHighRisk,
    }
}

#[test]
fn wrong_parent_pin_does_not_change_guardian_policy() {
    let store = MemorySecretStore::default();
    store.put_secret("parent-pin", "2468").unwrap();
    let mut authorization = ParentAuthorization::new(store, "parent-pin");
    let mut guardian = GuardianPolicyStore::default();
    let before = guardian.current_parent_policy().clone();

    assert!(save_parent_policy_with_authorization(
        &mut authorization,
        &mut guardian,
        "1357",
        1_000,
        policy(),
    )
    .is_err());
    assert_eq!(guardian.current_parent_policy(), &before);
}

#[test]
fn correct_parent_pin_saves_policy_through_guardian() {
    let store = MemorySecretStore::default();
    store.put_secret("parent-pin", "2468").unwrap();
    let mut authorization = ParentAuthorization::new(store, "parent-pin");
    let mut guardian = GuardianPolicyStore::default();
    let expected = policy();

    save_parent_policy_with_authorization(
        &mut authorization,
        &mut guardian,
        "2468",
        1_000,
        expected.clone(),
    )
    .unwrap();

    assert_eq!(guardian.current_parent_policy(), &expected);
}
