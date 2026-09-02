use policy_core::{evaluate_download, DownloadContext, DownloadMode, PolicyDecision};

#[test]
fn safe_downloads_are_allowed_in_both_modes() {
    for mode in [DownloadMode::BlockHighRisk, DownloadMode::RequireParentHighRisk] {
        let context = DownloadContext::new("worksheet.pdf", "application/pdf", 10)
            .with_download_mode(mode);
        assert_eq!(evaluate_download(&context), PolicyDecision::Allow);
    }
}

#[test]
fn block_high_risk_mode_blocks_double_extension_executables() {
    let context = DownloadContext::new("photo.jpg.exe", "application/octet-stream", 14)
        .with_download_mode(DownloadMode::BlockHighRisk);
    assert_eq!(evaluate_download(&context), PolicyDecision::Block);
}

#[test]
fn parent_gate_mode_gates_double_extension_executables() {
    let context = DownloadContext::new("photo.jpg.exe", "application/octet-stream", 14)
        .with_download_mode(DownloadMode::RequireParentHighRisk);
    assert_eq!(evaluate_download(&context), PolicyDecision::RequireParent);
}

#[test]
fn executable_mime_is_high_risk_even_with_safe_filename() {
    let context = DownloadContext::new(
        "homework.pdf",
        "application/vnd.microsoft.portable-executable",
        14,
    )
    .with_download_mode(DownloadMode::BlockHighRisk);
    assert_eq!(evaluate_download(&context), PolicyDecision::Block);
}

#[test]
fn archive_inspection_metadata_blocks_disguised_archive_in_block_mode() {
    let context = DownloadContext::new("game.exe.zip", "application/zip", 14)
        .with_download_mode(DownloadMode::BlockHighRisk)
        .with_archive_contains_high_risk(true);
    assert_eq!(evaluate_download(&context), PolicyDecision::Block);
}

#[test]
fn archive_inspection_metadata_parent_gates_disguised_archive_in_gate_mode() {
    let context = DownloadContext::new("game.exe.zip", "application/zip", 14)
        .with_download_mode(DownloadMode::RequireParentHighRisk)
        .with_archive_contains_high_risk(true);
    assert_eq!(evaluate_download(&context), PolicyDecision::RequireParent);
}

#[test]
fn archive_display_name_alone_does_not_mark_archive_high_risk() {
    let context = DownloadContext::new("game.exe.zip", "application/zip", 14)
        .with_download_mode(DownloadMode::BlockHighRisk)
        .with_archive_contains_high_risk(false);
    assert_eq!(evaluate_download(&context), PolicyDecision::Allow);
}

#[test]
fn explicit_parent_block_still_wins() {
    let context = DownloadContext::new("worksheet.pdf", "application/pdf", 14)
        .with_download_mode(DownloadMode::RequireParentHighRisk)
        .with_parent_blocked(true);
    assert_eq!(evaluate_download(&context), PolicyDecision::Block);
}

#[test]
fn explicit_parent_approval_allows_parent_gated_high_risk_download() {
    let context = DownloadContext::new("tool.exe", "application/octet-stream", 16)
        .with_download_mode(DownloadMode::RequireParentHighRisk)
        .with_parent_allowed(true);
    assert_eq!(evaluate_download(&context), PolicyDecision::Allow);
}

#[test]
fn explicit_parent_approval_cannot_override_block_high_risk_mode() {
    let context = DownloadContext::new("tool.exe", "application/octet-stream", 16)
        .with_download_mode(DownloadMode::BlockHighRisk)
        .with_parent_allowed(true);
    assert_eq!(evaluate_download(&context), PolicyDecision::Block);
}
