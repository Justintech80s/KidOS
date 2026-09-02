use crate::model::{DownloadContext, NavigationContext, PolicyDecision, RiskLevel, SiteCategory};

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

pub fn evaluate_download(ctx: &DownloadContext) -> PolicyDecision {
    if ctx.parent_blocked {
        return PolicyDecision::Block;
    }

    let name = ctx.file_name.to_ascii_lowercase();
    let mime = ctx.mime_type.to_ascii_lowercase();
    let high_risk_extension = [".exe", ".msi", ".bat", ".cmd", ".ps1", ".scr", ".com", ".js", ".vbs"]
        .iter()
        .any(|extension| name.ends_with(extension));
    let high_risk_mime = mime.contains("portable-executable")
        || mime.contains("x-msdownload")
        || mime.contains("x-msi")
        || mime.contains("x-bat");

    if high_risk_extension || high_risk_mime {
        return PolicyDecision::RequireParent;
    }

    if ctx.parent_allowed {
        return PolicyDecision::Allow;
    }

    PolicyDecision::RequireParent
}
