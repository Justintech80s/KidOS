use crate::model::{
    DownloadContext, DownloadMode, NavigationContext, PolicyDecision, RiskLevel, SiteCategory,
};

pub fn evaluate_navigation(ctx: &NavigationContext) -> PolicyDecision {
    if ctx.parent_blocked {
        return PolicyDecision::Block;
    }

    if ctx.parent_allowed && ctx.risk != RiskLevel::High && ctx.category != SiteCategory::Prohibited {
        return PolicyDecision::Allow;
    }

    if ctx.category == SiteCategory::Prohibited {
        return PolicyDecision::Block;
    }

    if ctx.category == SiteCategory::Unknown && ctx.risk == RiskLevel::High {
        return PolicyDecision::RequireParent;
    }

    if ctx.category == SiteCategory::Unknown {
        if ctx.age >= 13 && ctx.unknown_web_enabled {
            return PolicyDecision::Allow;
        }
        return PolicyDecision::RequireParent;
    }

    if matches!(ctx.category, SiteCategory::Approved | SiteCategory::Educational) {
        return PolicyDecision::Allow;
    }

    PolicyDecision::RequireParent
}

fn has_high_risk_final_extension(file_name: &str) -> bool {
    const HIGH_RISK_EXTENSIONS: [&str; 9] = [
        "exe", "msi", "bat", "cmd", "ps1", "scr", "com", "js", "vbs",
    ];

    let normalized = file_name
        .trim()
        .trim_end_matches(['.', ' '])
        .to_ascii_lowercase();

    normalized
        .rsplit_once('.')
        .map(|(_, extension)| HIGH_RISK_EXTENSIONS.contains(&extension.trim()))
        .unwrap_or(false)
}

fn has_high_risk_mime(mime_type: &str) -> bool {
    let mime = mime_type.to_ascii_lowercase();
    mime.contains("portable-executable")
        || mime.contains("x-msdownload")
        || mime.contains("x-msi")
        || mime.contains("x-bat")
}

pub fn evaluate_download(ctx: &DownloadContext) -> PolicyDecision {
    if ctx.parent_blocked {
        return PolicyDecision::Block;
    }

    let high_risk = has_high_risk_final_extension(&ctx.file_name)
        || has_high_risk_mime(&ctx.mime_type)
        || ctx.archive_contains_high_risk;

    if !high_risk {
        return PolicyDecision::Allow;
    }

    match ctx.download_mode {
        DownloadMode::BlockHighRisk => PolicyDecision::Block,
        DownloadMode::RequireParentHighRisk if ctx.parent_allowed => PolicyDecision::Allow,
        DownloadMode::RequireParentHighRisk => PolicyDecision::RequireParent,
    }
}
