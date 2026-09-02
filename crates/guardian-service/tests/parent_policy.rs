use guardian_service::{
    GuardianActor, GuardianError, GuardianPolicyStore, ParentDownloadMode, ParentPolicyConfig,
    SocialAccessMode, SocialAccessRule,
};

fn teen_policy() -> ParentPolicyConfig {
    ParentPolicyConfig {
        child_age: 15,
        allow_domains: vec!["khanacademy.org".into()],
        block_domains: vec!["unsafe.example".into()],
        teen_unknown_web_enabled: true,
        social_access: vec![SocialAccessRule {
            service: "youtube".into(),
            mode: SocialAccessMode::TimeLimited,
            start_minutes: Some(480),
            end_minutes: Some(1200),
        }],
        download_mode: ParentDownloadMode::RequireParentHighRisk,
    }
}

#[test]
fn child_actor_cannot_replace_parent_policy() {
    let mut store = GuardianPolicyStore::default();
    let before = store.current_parent_policy().clone();

    assert_eq!(
        store.replace_parent_policy(GuardianActor::Child, teen_policy()),
        Err(GuardianError::UnauthorizedRequest)
    );
    assert_eq!(store.current_parent_policy(), &before);
}

#[test]
fn authorized_parent_can_replace_parent_policy() {
    let mut store = GuardianPolicyStore::default();
    let policy = teen_policy();

    store
        .replace_parent_policy(GuardianActor::ParentAuthorized, policy.clone())
        .unwrap();

    assert_eq!(store.current_parent_policy(), &policy);
}

#[test]
fn guardian_rejects_unknown_web_for_non_teen_profiles() {
    let mut store = GuardianPolicyStore::default();
    let mut invalid = teen_policy();
    invalid.child_age = 10;

    assert!(matches!(
        store.replace_parent_policy(GuardianActor::ParentAuthorized, invalid),
        Err(GuardianError::InvalidPolicyInput(_))
    ));
}

#[test]
fn guardian_rejects_invalid_social_time_window() {
    let mut store = GuardianPolicyStore::default();
    let mut invalid = teen_policy();
    invalid.social_access[0].start_minutes = Some(1200);
    invalid.social_access[0].end_minutes = Some(480);

    assert!(matches!(
        store.replace_parent_policy(GuardianActor::ParentAuthorized, invalid),
        Err(GuardianError::InvalidPolicyInput(_))
    ));
}
