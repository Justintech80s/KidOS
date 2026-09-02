use guardian_service::{load_service_state, GuardianMode, ParentDownloadMode, ParentPolicyConfig};
use policy_core::{
    evaluate_download as policy_evaluate_download,
    evaluate_navigation as policy_evaluate_navigation,
    DownloadContext, DownloadMode, NavigationContext, PolicyDecision,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspacePlan {
    pub kind: String,
    pub title: String,
    pub capabilities: Vec<String>,
}

fn decision_name(decision: PolicyDecision) -> &'static str {
    match decision {
        PolicyDecision::Allow => "allow",
        PolicyDecision::Block => "block",
        PolicyDecision::RequireParent => "require_parent",
    }
}

fn host_from_url(url: &str) -> &str {
    let without_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(without_scheme)
        .split(':')
        .next()
        .unwrap_or(without_scheme)
}

fn domain_matches(host: &str, rule: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    let rule = rule.trim().trim_end_matches('.').to_ascii_lowercase();
    host == rule || host.ends_with(&format!(".{rule}"))
}

pub fn plan_workspace_impl(prompt: &str) -> WorkspacePlan {
    let normalized = prompt.to_ascii_lowercase();

    if ["code", "coding", "game", "program"]
        .iter()
        .any(|term| normalized.contains(term))
    {
        return WorkspacePlan {
            kind: "beginner_coding".into(),
            title: "Beginner Coding".into(),
            capabilities: vec!["beginner_coding".into(), "export_project".into()],
        };
    }

    if [
        "draw",
        "picture",
        "poster",
        "presentation",
        "slides",
        "cartoon",
    ]
    .iter()
    .any(|term| normalized.contains(term))
    {
        return WorkspacePlan {
            kind: "drawing_presentation".into(),
            title: "Draw & Present".into(),
            capabilities: vec!["drawing_presentation".into(), "export_project".into()],
        };
    }

    WorkspacePlan {
        kind: "story".into(),
        title: if normalized.contains("space") {
            "Space Story".into()
        } else {
            "Story".into()
        },
        capabilities: vec!["story".into(), "export_project".into()],
    }
}

pub fn evaluate_navigation_impl(url: &str) -> &'static str {
    decision_name(policy_evaluate_navigation(&NavigationContext::new(url, 10)))
}

pub fn evaluate_navigation_with_policy_impl(
    url: &str,
    policy: &ParentPolicyConfig,
) -> &'static str {
    let host = host_from_url(url);
    let parent_blocked = policy
        .block_domains
        .iter()
        .any(|rule| domain_matches(host, rule));
    let parent_allowed = policy
        .allow_domains
        .iter()
        .any(|rule| domain_matches(host, rule));

    let context = NavigationContext::new(host, policy.child_age)
        .with_parent_blocked(parent_blocked)
        .with_parent_allowed(parent_allowed)
        .with_unknown_web_enabled(policy.teen_unknown_web_enabled);

    decision_name(policy_evaluate_navigation(&context))
}

pub fn evaluate_download_impl(file_name: &str, mime_type: &str) -> &'static str {
    decision_name(policy_evaluate_download(&DownloadContext::new(
        file_name, mime_type, 10,
    )))
}

pub fn evaluate_download_with_policy_impl(
    file_name: &str,
    mime_type: &str,
    archive_contains_high_risk: bool,
    parent_allowed: bool,
    policy: &ParentPolicyConfig,
) -> &'static str {
    let download_mode = match policy.download_mode {
        ParentDownloadMode::BlockHighRisk => DownloadMode::BlockHighRisk,
        ParentDownloadMode::RequireParentHighRisk => DownloadMode::RequireParentHighRisk,
    };

    let context = DownloadContext::new(file_name, mime_type, policy.child_age)
        .with_parent_allowed(parent_allowed)
        .with_download_mode(download_mode)
        .with_archive_contains_high_risk(archive_contains_high_risk);

    decision_name(policy_evaluate_download(&context))
}

pub fn get_guardian_status_impl() -> &'static str {
    match load_service_state(None, None).mode {
        GuardianMode::Healthy => "healthy",
        GuardianMode::RestrictedSafeMode => "restricted_safe_mode",
    }
}

#[tauri::command]
pub fn plan_workspace(prompt: String) -> WorkspacePlan {
    plan_workspace_impl(&prompt)
}

#[tauri::command]
pub fn evaluate_navigation(url: String) -> String {
    evaluate_navigation_impl(&url).to_string()
}

#[tauri::command]
pub fn evaluate_download(file_name: String, mime_type: String) -> String {
    evaluate_download_impl(&file_name, &mime_type).to_string()
}

#[tauri::command]
pub fn get_guardian_status() -> String {
    get_guardian_status_impl().to_string()
}
