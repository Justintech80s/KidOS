use kidos_shell_lib::commands::{
    evaluate_download_impl, evaluate_navigation_impl, get_guardian_status_impl, plan_workspace_impl,
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
