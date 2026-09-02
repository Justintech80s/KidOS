use policy_core::{evaluate_download, DownloadContext, DownloadMode, PolicyDecision};

#[test]
fn block_all_mode_blocks_even_safe_downloads() {
    let context = DownloadContext::new("worksheet.pdf", "application/pdf", 10)
        .with_download_mode(DownloadMode::BlockAll);
    assert_eq!(evaluate_download(&context), PolicyDecision::Block);
}

#[test]
fn require_parent_mode_gates_safe_downloads() {
    let context = DownloadContext::new("worksheet.pdf", "application/pdf", 10)
        .with_download_mode(DownloadMode::RequireParent);
    assert_eq!(evaluate_download(&context), PolicyDecision::RequireParent);
}

#[test]
fn allow_safe_mode_allows_safe_downloads() {
    let context = DownloadContext::new("worksheet.pdf", "application/pdf", 14)
        .with_download_mode(DownloadMode::AllowSafe);
    assert_eq!(evaluate_download(&context), PolicyDecision::Allow);
}

#[test]
fn dangerous_double_extension_requires_parent() {
    let context = DownloadContext::new("family-photo.jpg.exe", "application/octet-stream", 14)
        .with_download_mode(DownloadMode::AllowSafe);
    assert_eq!(evaluate_download(&context), PolicyDecision::RequireParent);
}

#[test]
fn suspicious_embedded_executable_extension_requires_parent() {
    let context = DownloadContext::new("game.exe.jpg", "image/jpeg", 14)
        .with_download_mode(DownloadMode::AllowSafe);
    assert_eq!(evaluate_download(&context), PolicyDecision::RequireParent);
}

#[test]
fn executable_mime_requires_parent_even_with_safe_filename() {
    let context = DownloadContext::new(
        "homework.pdf",
        "application/vnd.microsoft.portable-executable",
        14,
    )
    .with_download_mode(DownloadMode::AllowSafe);
    assert_eq!(evaluate_download(&context), PolicyDecision::RequireParent);
}
