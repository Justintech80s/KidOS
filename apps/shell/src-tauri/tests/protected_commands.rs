use guardian_service::{GuardianPolicyStore, ParentDownloadMode, ParentPolicyConfig};
use kidos_shell_lib::commands::{
    evaluate_download_impl, evaluate_download_with_policy_impl, evaluate_navigation_impl,
    evaluate_navigation_with_policy_impl, get_guardian_status_impl, plan_workspace_impl,
};

#[test]
fn story_prompt_returns_story_workspace_without_extra_capabilities() {
    let plan = plan_workspace_impl("make a story about space");
    assert_eq!(plan.kind, "story");
    assert_eq!(plan.capabilities, vec!["story", "export_project"]);
}

#[test]
fn unknown_navigation_fails_closed_to_parent_gate() {
    assert_eq!(evaluate_navigation_impl("https://unknown.example"), "require_parent");
}

#[test]
fn executable_download_requires_parent() {
    assert_eq!(
        evaluate_download_impl("setup.exe", "application/octet-stream"),
        "require_parent"
    );
}

#[test]
fn guardian_without_loaded_policy_reports_restricted_safe_mode() {
    assert_eq!(get_guardian_status_impl(), "restricted_safe_mode");
}

#[test]
fn saved_parent_blocked_domain_blocks_navigation() {
    let mut policy = ParentPolicyConfig::default();
    policy.child_age = 15;
    policy.block_domains = vec!["blocked.example".into()];

    assert_eq!(
        evaluate_navigation_with_policy_impl("https://sub.blocked.example/path", &policy),
        "block"
    );
}

#[test]
fn saved_parent_allowed_domain_allows_navigation() {
    let mut policy = ParentPolicyConfig::default();
    policy.child_age = 10;
    policy.allow_domains = vec!["school.example".into()];

    assert_eq!(
        evaluate_navigation_with_policy_impl("https://school.example/lesson", &policy),
        "allow"
    );
}

#[test]
fn teen_unknown_web_setting_controls_unknown_navigation() {
    let mut policy = ParentPolicyConfig::default();
    policy.child_age = 15;
    policy.teen_unknown_web_enabled = true;

    assert_eq!(
        evaluate_navigation_with_policy_impl("https://newsite.example", &policy),
        "allow"
    );
}

#[test]
fn saved_block_high_risk_download_mode_blocks_executables() {
    let mut policy = ParentPolicyConfig::default();
    policy.download_mode = ParentDownloadMode::BlockHighRisk;

    assert_eq!(
        evaluate_download_with_policy_impl(
            "setup.exe",
            "application/octet-stream",
            false,
            false,
            &policy,
        ),
        "block"
    );
}

#[test]
fn saved_parent_gate_download_mode_gates_inspected_risky_archives() {
    let policy = GuardianPolicyStore::default().current_parent_policy().clone();

    assert_eq!(
        evaluate_download_with_policy_impl(
            "game.exe.zip",
            "application/zip",
            true,
            false,
            &policy,
        ),
        "require_parent"
    );
}
