use policy_core::{
    evaluate_download, evaluate_navigation, DownloadContext, NavigationContext, PolicyDecision,
    RiskLevel, SiteCategory,
};

#[test]
fn blocks_explicit_parent_denied_domain() {
    let ctx = NavigationContext::new("example-bad.test", 10).with_parent_blocked(true);
    assert_eq!(evaluate_navigation(&ctx), PolicyDecision::Block);
}

#[test]
fn unknown_high_risk_navigation_requires_parent() {
    let ctx = NavigationContext::new("unknown.test", 10)
        .with_category(SiteCategory::Unknown)
        .with_risk(RiskLevel::High);
    assert_eq!(evaluate_navigation(&ctx), PolicyDecision::RequireParent);
}

#[test]
fn executable_download_is_parent_gated() {
    let ctx = DownloadContext::new(
        "setup.exe",
        "application/vnd.microsoft.portable-executable",
        12,
    );
    assert_eq!(evaluate_download(&ctx), PolicyDecision::RequireParent);
}

#[test]
fn explicit_parent_block_wins_over_allow() {
    let ctx = NavigationContext::new("mixed.test", 15)
        .with_parent_allowed(true)
        .with_parent_blocked(true)
        .with_category(SiteCategory::Approved);
    assert_eq!(evaluate_navigation(&ctx), PolicyDecision::Block);
}

#[test]
fn young_child_unknown_navigation_requires_parent() {
    let ctx = NavigationContext::new("new-site.test", 9).with_category(SiteCategory::Unknown);
    assert_eq!(evaluate_navigation(&ctx), PolicyDecision::RequireParent);
}

#[test]
fn teen_unknown_navigation_can_be_enabled_explicitly() {
    let ctx = NavigationContext::new("new-site.test", 15)
        .with_category(SiteCategory::Unknown)
        .with_unknown_web_enabled(true);
    assert_eq!(evaluate_navigation(&ctx), PolicyDecision::Allow);
}

#[test]
fn prohibited_category_is_blocked() {
    let ctx = NavigationContext::new("gambling.test", 16).with_category(SiteCategory::Prohibited);
    assert_eq!(evaluate_navigation(&ctx), PolicyDecision::Block);
}

#[test]
fn approved_destination_is_allowed() {
    let ctx = NavigationContext::new("school.test", 10).with_category(SiteCategory::Educational);
    assert_eq!(evaluate_navigation(&ctx), PolicyDecision::Allow);
}
