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

fn has_suspicious_extension(file_name: &str) -> bool {
    const HIGH_RISK_EXTENSIONS: [&str; 9] = [
        "exe", "msi", "bat", "cmd", "ps1", "scr", "com", "js", "vbs",
    ];

    let normalized = file_name
        .trim()
        .trim_end_matches(['.', ' '])
        .to_ascii_lowercase();

    normalized
        .split('.')
        .skip(1)
        .any(|part| HIGH_RISK_EXTENSIONS.contains(&part.trim()))
}

pub fn evaluate_download(ctx: &DownloadContext) -> PolicyDecision {
    if ctx.parent_blocked {
        return PolicyDecision::Block;
    }

    let mime = ctx.mime_type.to_ascii_lowercase();
    let high_risk_mime = mime.contains("portable-executable")
        || mime.contains("x-msdownload")
        || mime.contains("x-msi")
        || mime.contains("x-bat");

    if has_suspicious_extension(&ctx.file_name) || high_risk_mime {
        return PolicyDecision::RequireParent;
    }

    if ctx.parent_allowed {
        return PolicyDecision::Allow;
    }

    match ctx.download_mode {
        DownloadMode::BlockAll => PolicyDecision::Block,
        DownloadMode::RequireParent => PolicyDecision::RequireParent,
        DownloadMode::AllowSafe => PolicyDecision::Allow,
    }
}
