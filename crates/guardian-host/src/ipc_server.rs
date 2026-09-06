#[cfg(target_os = "windows")]
use guardian_service::{
    GuardianActor, GuardianPolicyStore, ParentPolicyConfig,
    privileged_ipc::{
        decode_privileged_request, IpcAccountRole, PrivilegedNonceTracker, PrivilegedRequest,
        PrivilegedResponse, GUARDIAN_PIPE_NAME, MAX_IPC_MESSAGE_BYTES,
    },
    windows_lockdown::{
        AccountRole, ApprovedApp, LockdownInspection, LockdownProfile, WindowsAssignedAccessAdapter,
        WindowsLockdownAdapter, WindowsLockdownService,
    },
};
#[cfg(target_os = "windows")]
use std::{
    fs,
    mem::size_of,
    path::PathBuf,
    ptr::null_mut,
    time::{SystemTime, UNIX_EPOCH},
};
#[cfg(target_os = "windows")]
use secure_store::{ParentAuthorization, ParentAuthorizationResult, SecretStore, WindowsSecretStore};
#[cfg(target_os = "windows")]
use base64::Engine;
#[cfg(target_os = "windows")]
use policy_core::{
    evaluate_download as evaluate_download_policy, evaluate_media as evaluate_media_policy,
    DownloadContext, DownloadMode, MediaCategory, MediaContext, MediaRisk, PolicyDecision,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, ERROR_PIPE_CONNECTED, INVALID_HANDLE_VALUE},
    NetworkManagement::NetManagement::{
        NetApiBufferFree, NetUserGetInfo, USER_INFO_1, USER_PRIV_ADMIN, USER_PRIV_GUEST,
        USER_PRIV_USER, NERR_Success,
    },
    Security::{Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW, SECURITY_ATTRIBUTES},
    Storage::FileSystem::{FlushFileBuffers, ReadFile, WriteFile},
    System::{
        Memory::LocalFree,
        Pipes::{
            ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_ACCESS_DUPLEX,
            PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE, PIPE_WAIT,
        },
    },
};

#[cfg(target_os = "windows")]
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

#[cfg(target_os = "windows")]
const PARENT_PIN_KEY: &str = "parent-pin";

#[cfg(target_os = "windows")]
fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(target_os = "windows")]
fn guardian_data_dir() -> Result<PathBuf, String> {
    let base = std::env::var_os("PROGRAMDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "Guardian could not resolve ProgramData.".to_string())?;
    Ok(base.join("KidOS").join("Guardian"))
}


#[cfg(target_os = "windows")]
fn recovery_marker_path() -> Result<PathBuf, String> {
    Ok(guardian_data_dir()?.join("recovery-required.flag"))
}

#[cfg(target_os = "windows")]
fn lockdown_backup_path() -> Result<PathBuf, String> {
    Ok(guardian_data_dir()?.join("last-lockdown-profile.json"))
}

#[cfg(target_os = "windows")]
fn persist_lockdown_profile(profile: &guardian_service::privileged_ipc::IpcLockdownProfile) -> Result<(), String> {
    let dir = guardian_data_dir()?;
    fs::create_dir_all(&dir).map_err(|_| "Guardian could not create recovery storage.".to_string())?;
    let bytes = serde_json::to_vec_pretty(profile).map_err(|_| "Guardian could not encode recovery profile.".to_string())?;
    fs::write(lockdown_backup_path()?, bytes).map_err(|_| "Guardian could not save recovery profile.".to_string())
}

#[cfg(target_os = "windows")]
fn mark_recovery_required(reason: &str) {
    if let Ok(path) = recovery_marker_path() {
        let _ = fs::write(path, reason.as_bytes());
    }
}

#[cfg(target_os = "windows")]
fn clear_recovery_marker() {
    if let Ok(path) = recovery_marker_path() {
        let _ = fs::remove_file(path);
    }
}

#[cfg(target_os = "windows")]
fn pin_marker_path() -> Result<PathBuf, String> {
    Ok(guardian_data_dir()?.join("parent-pin.initialized"))
}

#[cfg(target_os = "windows")]
fn policy_path() -> Result<PathBuf, String> {
    Ok(guardian_data_dir()?.join("parent-policy.json"))
}

#[cfg(target_os = "windows")]
fn pin_is_initialized() -> bool {
    pin_marker_path().map(|path| path.exists()).unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn mark_pin_initialized() -> Result<(), String> {
    let dir = guardian_data_dir()?;
    fs::create_dir_all(&dir).map_err(|_| "Guardian could not create its protected data folder.".to_string())?;
    fs::write(pin_marker_path()?, b"1").map_err(|_| "Guardian could not record parent PIN initialization.".to_string())
}

#[cfg(target_os = "windows")]
fn load_parent_policy() -> ParentPolicyConfig {
    let Ok(path) = policy_path() else { return ParentPolicyConfig::default(); };
    let Ok(contents) = fs::read_to_string(&path) else { return ParentPolicyConfig::default(); };
    match serde_json::from_str(&contents) {
        Ok(policy) => policy,
        Err(_) => {
            let corrupt = path.with_extension(format!("corrupt-{}.json", now_seconds()));
            let _ = fs::rename(&path, corrupt);
            let _ = fs::write(
                guardian_data_dir().unwrap_or_else(|_| PathBuf::from(r"C:\ProgramData\KidOS\Guardian")).join("recovery-required.flag"),
                b"parent-policy-corrupt",
            );
            ParentPolicyConfig::default()
        }
    }
}

#[cfg(target_os = "windows")]
fn persist_parent_policy(policy: &ParentPolicyConfig) -> Result<(), String> {
    let dir = guardian_data_dir()?;
    fs::create_dir_all(&dir).map_err(|_| "Guardian could not create its protected data folder.".to_string())?;
    let path = policy_path()?;
    let temp = path.with_extension("json.tmp");
    let encoded = serde_json::to_vec_pretty(policy)
        .map_err(|_| "Guardian could not encode parent policy.".to_string())?;
    fs::write(&temp, encoded).map_err(|_| "Guardian could not save parent policy.".to_string())?;
    fs::rename(&temp, &path).map_err(|_| "Guardian could not finalize parent policy.".to_string())?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn verify_parent(
    authorization: &mut ParentAuthorization<WindowsSecretStore>,
    pin: &str,
) -> Result<ParentAuthorizationResult, String> {
    authorization
        .verify(pin, now_seconds())
        .map_err(|_| "Guardian could not verify the parent PIN.".to_string())
}

#[cfg(target_os = "windows")]
fn verification_response(result: ParentAuthorizationResult) -> PrivilegedResponse {
    match result {
        ParentAuthorizationResult::Authorized => PrivilegedResponse::ParentVerification { authorized: true, locked: false },
        ParentAuthorizationResult::Denied => PrivilegedResponse::ParentVerification { authorized: false, locked: false },
        ParentAuthorizationResult::Locked => PrivilegedResponse::ParentVerification { authorized: false, locked: true },
    }
}

#[cfg(target_os = "windows")]
fn error_response(code: impl Into<String>, message: impl Into<String>) -> PrivilegedResponse {
    PrivilegedResponse::Error { code: code.into(), message: message.into() }
}


#[cfg(target_os = "windows")]
fn classifier_healthy() -> bool {
    let token = match std::env::var("KIDOS_MEDIA_CLASSIFIER_TOKEN") {
        Ok(value) => value,
        Err(_) => {
            let path = std::env::var("KIDOS_MEDIA_TOKEN_FILE")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(r"C:\ProgramData\KidOS\Guardian\media-classifier.token"));
            match fs::read_to_string(path) {
                Ok(value) => value.trim().to_string(),
                Err(_) => return false,
            }
        }
    };
    if token.len() < 32 {
        return false;
    }
    let endpoint = std::env::var("KIDOS_MEDIA_CLASSIFIER_HEALTH_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8765/health".into());
    let client = match reqwest::blocking::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(1))
        .timeout(std::time::Duration::from_secs(4))
        .build()
    {
        Ok(client) => client,
        Err(_) => return false,
    };
    match client.get(endpoint).header("x-kidos-classifier-token", token).send() {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

#[cfg(target_os = "windows")]
fn policy_file_valid() -> bool {
    let Ok(path) = policy_path() else { return false; };
    if !path.exists() {
        return true;
    }
    let Ok(contents) = fs::read_to_string(path) else { return false; };
    serde_json::from_str::<ParentPolicyConfig>(&contents).is_ok()
}

#[cfg(target_os = "windows")]
fn recovery_reason() -> Option<String> {
    let path = recovery_marker_path().ok()?;
    fs::read_to_string(path).ok().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

#[cfg(target_os = "windows")]
fn reset_parent_policy_to_defaults(
    parent_policy: &mut GuardianPolicyStore,
) -> Result<(), String> {
    let policy = ParentPolicyConfig::default();
    parent_policy
        .replace_parent_policy(GuardianActor::ParentAuthorized, policy.clone())
        .map_err(|error| error.to_string())?;
    persist_parent_policy(&policy)?;
    clear_recovery_marker();
    Ok(())
}

#[cfg(target_os = "windows")]
fn validate_standard_windows_account(account: &str) -> Result<(), String> {
    let account = account.trim();
    if account.is_empty() || account.len() > 256 {
        return Err("Windows child account name is invalid.".into());
    }

    let account_wide = wide(account);
    let mut buffer = null_mut();
    let status = unsafe { NetUserGetInfo(null_mut(), account_wide.as_ptr(), 1, &mut buffer) };
    if status != NERR_Success {
        return Err(format!("Windows child account '{account}' was not found."));
    }

    let info = unsafe { &*(buffer as *const USER_INFO_1) };
    let privilege = info.usri1_priv;
    unsafe { NetApiBufferFree(buffer) };

    match privilege {
        USER_PRIV_USER => Ok(()),
        USER_PRIV_ADMIN => Err("Windows Lockdown Mode cannot use an administrator account.".into()),
        USER_PRIV_GUEST => Err("Windows Lockdown Mode cannot use a guest account.".into()),
        _ => Err("KidOS could not confirm that the selected Windows account is a standard user.".into()),
    }
}

#[cfg(target_os = "windows")]
fn prohibited_executable_name(path: &std::path::Path) -> bool {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    matches!(
        name.as_str(),
        "cmd.exe"
            | "powershell.exe"
            | "pwsh.exe"
            | "regedit.exe"
            | "wt.exe"
            | "wscript.exe"
            | "cscript.exe"
            | "mmc.exe"
            | "taskmgr.exe"
            | "reg.exe"
            | "schtasks.exe"
            | "sc.exe"
            | "net.exe"
            | "net1.exe"
            | "mshta.exe"
            | "rundll32.exe"
            | "control.exe"
            | "computerdefaults.exe"
            | "eventvwr.exe"
    )
}

#[cfg(target_os = "windows")]
fn validate_approved_executable(app: &guardian_service::privileged_ipc::IpcApprovedApp) -> Result<String, String> {
    let raw = app.executable_path.trim();
    if raw.is_empty() || raw.len() > 1024 {
        return Err(format!("Approved app '{}' has an invalid executable path.", app.display_name));
    }

    let path = std::path::Path::new(raw);
    if !path.is_absolute() {
        return Err(format!("Approved app '{}' must use an absolute executable path.", app.display_name));
    }
    if path.extension().and_then(|value| value.to_str()).map(|value| !value.eq_ignore_ascii_case("exe")).unwrap_or(true) {
        return Err(format!("Approved app '{}' must point to a .exe file.", app.display_name));
    }
    if prohibited_executable_name(path) {
        return Err(format!("Approved app '{}' is a prohibited Windows administrative/system tool.", app.display_name));
    }

    let metadata = fs::metadata(path)
        .map_err(|_| format!("Approved app '{}' does not exist at the selected path.", app.display_name))?;
    if !metadata.is_file() {
        return Err(format!("Approved app '{}' path is not a file.", app.display_name));
    }

    let canonical = fs::canonicalize(path)
        .map_err(|_| format!("KidOS could not resolve the approved app '{}' path.", app.display_name))?;
    if prohibited_executable_name(&canonical) {
        return Err(format!("Approved app '{}' resolves to a prohibited Windows administrative/system tool.", app.display_name));
    }

    canonical
        .to_str()
        .map(|value| value.to_string())
        .ok_or_else(|| format!("Approved app '{}' path is not valid Unicode.", app.display_name))
}


#[cfg(target_os = "windows")]
fn guardian_download_decision(
    file_name: &str,
    mime_type: &str,
    archive_contains_high_risk: bool,
    policy: &ParentPolicyConfig,
) -> &'static str {
    let mode = match policy.download_mode {
        guardian_service::ParentDownloadMode::BlockHighRisk => DownloadMode::BlockHighRisk,
        guardian_service::ParentDownloadMode::RequireParentHighRisk => DownloadMode::RequireParentHighRisk,
    };

    let context = DownloadContext::new(file_name, mime_type, policy.child_age)
        .with_download_mode(mode)
        .with_archive_contains_high_risk(archive_contains_high_risk);

    match evaluate_download_policy(&context) {
        PolicyDecision::Allow => "allow",
        PolicyDecision::Block => "block",
        PolicyDecision::RequireParent => "require_parent",
    }
}


#[cfg(target_os = "windows")]
#[derive(serde::Serialize)]
struct ClassifierRequest<'a> {
    path: &'a str,
}

#[cfg(target_os = "windows")]
#[derive(serde::Deserialize)]
struct ClassifierResponse {
    category: String,
    risk: String,
    confidence: f32,
    high_confidence: bool,
    classifier_available: bool,
    frames_checked: u32,
}

#[cfg(target_os = "windows")]
fn classify_media_file_with_local_ai(path: &str) -> Result<ClassifierResponse, String> {
    let path_obj = std::path::Path::new(path);
    if !path_obj.is_absolute() || !path_obj.is_file() {
        return Err("KidOS classifier received an invalid media path.".into());
    }

    let token = match std::env::var("KIDOS_MEDIA_CLASSIFIER_TOKEN") {
        Ok(value) => value,
        Err(_) => {
            let path = std::env::var("KIDOS_MEDIA_TOKEN_FILE")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(r"C:\ProgramData\KidOS\Guardian\media-classifier.token"));
            fs::read_to_string(path)
                .map_err(|_| "KidOS media classifier token is not configured.".to_string())?
                .trim()
                .to_string()
        }
    };
    if token.len() < 32 {
        return Err("KidOS media classifier token is too short.".into());
    }

    let endpoint = std::env::var("KIDOS_MEDIA_CLASSIFIER_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8765/classify".into());

    let client = reqwest::blocking::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(2))
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .map_err(|_| "KidOS could not initialize its local classifier client.".to_string())?;

    let response = client
        .post(endpoint)
        .header("x-kidos-classifier-token", token)
        .json(&ClassifierRequest { path })
        .send()
        .map_err(|_| "KidOS local media classifier is unavailable.".to_string())?;

    if !response.status().is_success() {
        return Err(format!(
            "KidOS local media classifier returned HTTP {}.",
            response.status()
        ));
    }

    response
        .json::<ClassifierResponse>()
        .map_err(|_| "KidOS received an invalid classifier response.".to_string())
}

#[cfg(target_os = "windows")]
fn evaluate_classifier_response(
    file_name: &str,
    classified: &ClassifierResponse,
    policy: &ParentPolicyConfig,
) -> (&'static str, String, String, f32, bool, u32) {
    let category = match classified.category.as_str() {
        "safe" => MediaCategory::Safe,
        "adult_nudity" => MediaCategory::AdultNudity,
        "sexualized_content" => MediaCategory::SexualizedContent,
        "graphic_violence" => MediaCategory::GraphicViolence,
        "self_harm" => MediaCategory::SelfHarm,
        "drugs" => MediaCategory::Drugs,
        "extremist_content" => MediaCategory::ExtremistContent,
        "scam" => MediaCategory::Scam,
        _ => MediaCategory::Uncertain,
    };
    let risk = match classified.risk.as_str() {
        "low" => MediaRisk::Low,
        "medium" => MediaRisk::Medium,
        "high" => MediaRisk::High,
        _ => MediaRisk::High,
    };

    let context = MediaContext {
        age: policy.child_age,
        category,
        risk,
        high_confidence: classified.high_confidence,
        parent_blocked: false,
        classifier_available: classified.classifier_available,
        teen_uncertain_enabled: false,
    };

    let decision = match evaluate_media_policy(&context) {
        PolicyDecision::Allow => "allow",
        PolicyDecision::Block => "block",
        PolicyDecision::RequireParent => "require_parent",
    };

    (
        decision,
        classified.category.clone(),
        classified.risk.clone(),
        classified.confidence,
        classified.high_confidence,
        classified.frames_checked,
    )
}



#[cfg(target_os = "windows")]
#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
struct QuarantineAuditRecord {
    item_id: String,
    file_name: String,
    event: String,
    category: Option<String>,
    risk: Option<String>,
    confidence: Option<f32>,
    decision: Option<String>,
    reason: Option<String>,
    timestamp_seconds: u64,
}

#[cfg(target_os = "windows")]
fn quarantine_audit_path() -> Result<PathBuf, String> {
    Ok(guardian_data_dir()?.join("quarantine-audit.jsonl"))
}

#[cfg(target_os = "windows")]
fn append_quarantine_audit(record: &QuarantineAuditRecord) -> Result<(), String> {
    let dir = guardian_data_dir()?;
    fs::create_dir_all(&dir).map_err(|_| "KidOS could not create Guardian audit storage.".to_string())?;
    let line = serde_json::to_string(record)
        .map_err(|_| "KidOS could not encode quarantine audit entry.".to_string())?;
    use std::io::Write as _;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(quarantine_audit_path()?)
        .map_err(|_| "KidOS could not open quarantine audit log.".to_string())?;
    writeln!(file, "{line}")
        .map_err(|_| "KidOS could not append quarantine audit entry.".to_string())
}

#[cfg(target_os = "windows")]
fn latest_quarantine_audit(item_id: &str) -> Option<QuarantineAuditRecord> {
    let contents = fs::read_to_string(quarantine_audit_path().ok()?).ok()?;
    contents
        .lines()
        .filter_map(|line| serde_json::from_str::<QuarantineAuditRecord>(line).ok())
        .filter(|record| record.item_id == item_id)
        .last()
}

#[cfg(target_os = "windows")]
fn classifier_thumbnail(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let token = match std::env::var("KIDOS_MEDIA_CLASSIFIER_TOKEN") {
        Ok(value) => value,
        Err(_) => {
            let path = std::env::var("KIDOS_MEDIA_TOKEN_FILE")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(r"C:\ProgramData\KidOS\Guardian\media-classifier.token"));
            fs::read_to_string(path)
                .map_err(|_| "KidOS media classifier token is not configured.".to_string())?
                .trim()
                .to_string()
        }
    };
    if token.len() < 32 {
        return Err("KidOS media classifier token is invalid.".into());
    }

    let endpoint = std::env::var("KIDOS_MEDIA_CLASSIFIER_THUMBNAIL_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8765/thumbnail".into());
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(2))
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|_| "KidOS could not initialize its thumbnail client.".to_string())?;

    let response = client
        .post(endpoint)
        .header("x-kidos-classifier-token", token)
        .json(&ClassifierRequest { path: &path.to_string_lossy() })
        .send()
        .map_err(|_| "KidOS local media thumbnail service is unavailable.".to_string())?;

    if !response.status().is_success() {
        return Err(format!("KidOS thumbnail service returned HTTP {}.", response.status()));
    }

    let bytes = response.bytes()
        .map_err(|_| "KidOS could not read the media thumbnail.".to_string())?;
    if bytes.len() > 48 * 1024 {
        return Err("KidOS refused an oversized media thumbnail.".into());
    }
    Ok(bytes.to_vec())
}

#[cfg(target_os = "windows")]
fn quarantine_pending_dir() -> PathBuf {
    PathBuf::from(r"C:\ProgramData\KidOS\Quarantine\Pending")
}

#[cfg(target_os = "windows")]
fn quarantine_blocked_dir() -> PathBuf {
    PathBuf::from(r"C:\ProgramData\KidOS\Quarantine\Blocked")
}

#[cfg(target_os = "windows")]
fn quarantine_approved_dir() -> PathBuf {
    PathBuf::from(r"C:\ProgramData\KidOS\Quarantine\Approved")
}

#[cfg(target_os = "windows")]
fn safe_quarantine_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 220
        && !value.contains("..")
        && !value.contains('/')
        && !value.contains('\\')
        && value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_' | ' ' | '(' | ')'))
}

#[cfg(target_os = "windows")]
fn list_quarantine_items() -> Result<Vec<guardian_service::privileged_ipc::QuarantineItem>, String> {
    let directory = quarantine_pending_dir();
    fs::create_dir_all(&directory)
        .map_err(|_| "KidOS could not open the protected quarantine folder.".to_string())?;

    let mut items = Vec::new();
    for entry in fs::read_dir(&directory)
        .map_err(|_| "KidOS could not read the protected quarantine folder.".to_string())?
    {
        let entry = entry.map_err(|_| "KidOS could not read a quarantine item.".to_string())?;
        let metadata = entry.metadata().map_err(|_| "KidOS could not inspect a quarantine item.".to_string())?;
        if !metadata.is_file() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if !safe_quarantine_id(&id) {
            continue;
        }
        let file_name = id.splitn(3, '-').nth(2).unwrap_or(&id).to_string();
        let modified_seconds = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|value| value.as_secs())
            .unwrap_or(0);
        let audit = latest_quarantine_audit(&id);
        items.push(guardian_service::privileged_ipc::QuarantineItem {
            id,
            file_name,
            size_bytes: metadata.len(),
            modified_seconds,
            category: audit.as_ref().and_then(|record| record.category.clone()),
            risk: audit.as_ref().and_then(|record| record.risk.clone()),
            confidence: audit.as_ref().and_then(|record| record.confidence),
            reason: audit.as_ref().and_then(|record| record.reason.clone()),
        });
    }
    items.sort_by(|a, b| b.modified_seconds.cmp(&a.modified_seconds));
    Ok(items)
}

#[cfg(target_os = "windows")]
fn review_quarantine_item(item_id: &str, action: &str) -> Result<(), String> {
    if !safe_quarantine_id(item_id) {
        return Err("KidOS rejected an invalid quarantine item identifier.".into());
    }

    let source = quarantine_pending_dir().join(item_id);
    let canonical_parent = fs::canonicalize(quarantine_pending_dir())
        .map_err(|_| "KidOS quarantine directory is unavailable.".to_string())?;
    let canonical_source = fs::canonicalize(&source)
        .map_err(|_| "The quarantine item no longer exists.".to_string())?;
    if !canonical_source.starts_with(&canonical_parent) || !canonical_source.is_file() {
        return Err("KidOS rejected a quarantine path outside its protected folder.".into());
    }

    let result = match action {
        "approve" => {
            let target_dir = quarantine_approved_dir();
            fs::create_dir_all(&target_dir).map_err(|_| "KidOS could not create the approved-media folder.".to_string())?;
            fs::rename(&canonical_source, target_dir.join(item_id))
                .map_err(|_| "KidOS could not approve this quarantine item.".to_string())
        }
        "delete" => fs::remove_file(&canonical_source)
            .map_err(|_| "KidOS could not delete this quarantine item.".to_string()),
        "keep_blocked" => {
            let target_dir = quarantine_blocked_dir();
            fs::create_dir_all(&target_dir).map_err(|_| "KidOS could not create the blocked-media folder.".to_string())?;
            fs::rename(&canonical_source, target_dir.join(item_id))
                .map_err(|_| "KidOS could not retain this quarantine item as blocked.".to_string())
        }
        _ => Err("KidOS rejected an unknown quarantine review action.".into()),
    };

    if result.is_ok() {
        let existing = latest_quarantine_audit(item_id).unwrap_or_default();
        let _ = append_quarantine_audit(&QuarantineAuditRecord {
            item_id: item_id.to_string(),
            file_name: existing.file_name,
            event: format!("parent_{action}"),
            category: existing.category,
            risk: existing.risk,
            confidence: existing.confidence,
            decision: existing.decision,
            reason: Some(format!("Parent selected '{action}' in quarantine review.")),
            timestamp_seconds: now_seconds(),
        });
    }

    result
}

#[cfg(target_os = "windows")]
fn profile_from_ipc(profile: guardian_service::privileged_ipc::IpcLockdownProfile) -> Result<LockdownProfile, String> {
    validate_standard_windows_account(&profile.account)?;
    if profile.apps.is_empty() || profile.apps.len() > 64 {
        return Err("approved app list must contain 1 through 64 entries".into());
    }

    let account_role = match profile.account_role {
        IpcAccountRole::Standard => AccountRole::Standard,
        IpcAccountRole::Administrator => AccountRole::Administrator,
        IpcAccountRole::Unknown => AccountRole::Unknown,
    };

    let mut apps = Vec::with_capacity(profile.apps.len());
    for app in profile.apps {
        if app.id.trim().is_empty()
            || app.display_name.trim().is_empty()
            || app.executable_path.trim().is_empty()
            || app.executable_path.len() > 1024
        {
            return Err("approved application entry is invalid".into());
        }
        let executable_path = validate_approved_executable(&app)?;
        apps.push(ApprovedApp {
            id: app.id,
            display_name: app.display_name,
            executable_path,
        });
    }

    Ok(LockdownProfile {
        profile_id: profile.profile_id,
        account: profile.account,
        account_role,
        apps,
    })
}

#[cfg(target_os = "windows")]
fn current_platform_state() -> (String, Option<String>) {
    let adapter = WindowsAssignedAccessAdapter::default();
    match adapter.inspect() {
        Ok(LockdownInspection::Configured) => ("locked".into(), None),
        Ok(LockdownInspection::NotConfigured) => ("unmanaged".into(), None),
        Ok(LockdownInspection::Unsupported) => (
            "restricted_safe_mode".into(),
            Some("Windows Assigned Access provider is unsupported.".into()),
        ),
        Err(error) => (
            "restricted_safe_mode".into(),
            Some(format!("Guardian could not inspect Assigned Access: {error:?}")),
        ),
    }
}

#[cfg(target_os = "windows")]
fn handle_request(
    bytes: &[u8],
    nonce_tracker: &mut PrivilegedNonceTracker,
    lockdown_service: &mut WindowsLockdownService<WindowsAssignedAccessAdapter>,
    parent_authorization: &mut ParentAuthorization<WindowsSecretStore>,
    parent_policy: &mut GuardianPolicyStore,
) -> PrivilegedResponse {
    let envelope = match decode_privileged_request(bytes) {
        Ok(envelope) => envelope,
        Err(error) => return error_response("invalid_request", error),
    };

    if let Err(code) = nonce_tracker.accept(&envelope) {
        return error_response(code, "Guardian rejected the privileged request.");
    }

    match envelope.request {
        PrivilegedRequest::Status => {
            let (state, reason) = current_platform_state();
            PrivilegedResponse::Status { state, reason }
        }
        PrivilegedRequest::ConfigureParentPin { new_pin, current_pin } => {
            if !(4..=8).contains(&new_pin.len()) || !new_pin.chars().all(|ch| ch.is_ascii_digit()) {
                return error_response("invalid_parent_pin", "Parent PIN must contain 4 through 8 digits.");
            }

            if pin_is_initialized() {
                let Some(current_pin) = current_pin else {
                    return error_response("current_parent_pin_required", "Changing the parent PIN requires the current PIN.");
                };
                match verify_parent(parent_authorization, &current_pin) {
                    Ok(ParentAuthorizationResult::Authorized) => {}
                    Ok(ParentAuthorizationResult::Denied) => return error_response("parent_pin_denied", "Current parent PIN was not accepted."),
                    Ok(ParentAuthorizationResult::Locked) => return error_response("parent_pin_locked", "Parent PIN entry is temporarily locked."),
                    Err(error) => return error_response("parent_pin_error", error),
                }
            }

            let store = WindowsSecretStore::new("KidOSGuardian");
            if let Err(error) = store.put_secret(PARENT_PIN_KEY, &new_pin) {
                return error_response("parent_pin_store_failed", format!("Guardian could not protect the parent PIN: {error}"));
            }
            *parent_authorization = ParentAuthorization::new(WindowsSecretStore::new("KidOSGuardian"), PARENT_PIN_KEY);
            if let Err(error) = mark_pin_initialized() {
                return error_response("parent_pin_marker_failed", error);
            }
            PrivilegedResponse::Ack { message: "Parent PIN is protected by the KidOS Guardian service.".into() }
        }
        PrivilegedRequest::VerifyParentPin { pin } => {
            match verify_parent(parent_authorization, &pin) {
                Ok(result) => verification_response(result),
                Err(error) => error_response("parent_pin_error", error),
            }
        }
        PrivilegedRequest::SaveParentPolicy { pin, policy } => {
            match verify_parent(parent_authorization, &pin) {
                Ok(ParentAuthorizationResult::Authorized) => {}
                Ok(ParentAuthorizationResult::Denied) => return error_response("parent_pin_denied", "Parent PIN was not accepted."),
                Ok(ParentAuthorizationResult::Locked) => return error_response("parent_pin_locked", "Parent PIN entry is temporarily locked."),
                Err(error) => return error_response("parent_pin_error", error),
            }

            if let Err(error) = parent_policy.replace_parent_policy(GuardianActor::ParentAuthorized, policy.clone()) {
                return error_response("invalid_parent_policy", error.to_string());
            }
            if let Err(error) = persist_parent_policy(&policy) {
                return error_response("parent_policy_store_failed", error);
            }
            PrivilegedResponse::Ack { message: "Parent safety policy saved by Guardian.".into() }
        }
        PrivilegedRequest::GetParentPolicy => PrivilegedResponse::ParentPolicy {
            policy: parent_policy.current_parent_policy().clone(),
        },
        PrivilegedRequest::EvaluateDownload {
            url,
            file_name,
            mime_type,
            archive_contains_high_risk,
        } => {
            if !(url.starts_with("https://") || url.starts_with("http://")) {
                return error_response("invalid_download_url", "KidOS downloads must originate from http or https.");
            }
            if file_name.trim().is_empty() || file_name.len() > 260 {
                return error_response("invalid_download_name", "KidOS rejected an invalid download filename.");
            }
            let decision = guardian_download_decision(
                &file_name,
                &mime_type,
                archive_contains_high_risk,
                parent_policy.current_parent_policy(),
            );
            PrivilegedResponse::PolicyDecision { decision: decision.into() }
        }
        PrivilegedRequest::EvaluateMedia {
            file_name,
            category,
            risk,
            high_confidence,
            classifier_available,
        } => {
            if file_name.trim().is_empty() || file_name.len() > 260 {
                return error_response("invalid_media_name", "KidOS rejected an invalid media filename.");
            }

            let category = match category.as_str() {
                "safe" => MediaCategory::Safe,
                "adult_nudity" => MediaCategory::AdultNudity,
                "sexualized_content" => MediaCategory::SexualizedContent,
                "graphic_violence" => MediaCategory::GraphicViolence,
                "self_harm" => MediaCategory::SelfHarm,
                "drugs" => MediaCategory::Drugs,
                "extremist_content" => MediaCategory::ExtremistContent,
                "scam" => MediaCategory::Scam,
                _ => MediaCategory::Uncertain,
            };
            let risk = match risk.as_str() {
                "low" => MediaRisk::Low,
                "medium" => MediaRisk::Medium,
                "high" => MediaRisk::High,
                _ => MediaRisk::High,
            };
            let policy = parent_policy.current_parent_policy();
            let context = MediaContext {
                age: policy.child_age,
                category,
                risk,
                high_confidence,
                parent_blocked: false,
                classifier_available,
                teen_uncertain_enabled: false,
            };
            let decision = match evaluate_media_policy(&context) {
                PolicyDecision::Allow => "allow",
                PolicyDecision::Block => "block",
                PolicyDecision::RequireParent => "require_parent",
            };
            PrivilegedResponse::PolicyDecision { decision: decision.into() }
        }
        PrivilegedRequest::ClassifyMediaFile { path } => {
            let file_name = std::path::Path::new(&path)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("media")
                .to_string();

            let classified = match classify_media_file_with_local_ai(&path) {
                Ok(value) => value,
                Err(error) => {
                    return PrivilegedResponse::MediaClassification {
                        decision: "require_parent".into(),
                        category: "uncertain".into(),
                        risk: "high".into(),
                        confidence: 0.0,
                        high_confidence: false,
                        frames_checked: 0,
                    };
                }
            };

            let (decision, category, risk, confidence, high_confidence, frames_checked) =
                evaluate_classifier_response(
                    &file_name,
                    &classified,
                    parent_policy.current_parent_policy(),
                );

            let item_id = std::path::Path::new(&path)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or(&file_name)
                .to_string();
            let reason = match decision {
                "allow" => "AI classifier and Guardian policy approved this media.",
                "block" => "AI classifier and Guardian policy marked this media unsafe.",
                _ => "Media requires parent review because the result was uncertain or policy requires approval.",
            }.to_string();
            let _ = append_quarantine_audit(&QuarantineAuditRecord {
                item_id,
                file_name: file_name.clone(),
                event: "classified".into(),
                category: Some(category.clone()),
                risk: Some(risk.clone()),
                confidence: Some(confidence),
                decision: Some(decision.into()),
                reason: Some(reason),
                timestamp_seconds: now_seconds(),
            });

            PrivilegedResponse::MediaClassification {
                decision: decision.into(),
                category,
                risk,
                confidence,
                high_confidence,
                frames_checked,
            }
        }
        PrivilegedRequest::ListQuarantine { pin } => {
            match parent_authorization.verify(&pin, now_seconds()) {
                Ok(ParentAuthorizationResult::Authorized) => match list_quarantine_items() {
                    Ok(items) => PrivilegedResponse::QuarantineItems { items },
                    Err(message) => error_response("quarantine_list_failed", &message),
                },
                Ok(ParentAuthorizationResult::Locked) => error_response("parent_pin_locked", "Parent PIN verification is temporarily locked."),
                Ok(ParentAuthorizationResult::Denied) => error_response("parent_pin_denied", "Parent PIN was not accepted."),
                Err(_) => error_response("parent_pin_error", "KidOS could not verify the parent PIN."),
            }
        }
        PrivilegedRequest::PreviewQuarantine { pin, item_id } => {
            match parent_authorization.verify(&pin, now_seconds()) {
                Ok(ParentAuthorizationResult::Authorized) => {
                    if !safe_quarantine_id(&item_id) {
                        return error_response("invalid_quarantine_item", "KidOS rejected an invalid quarantine item identifier.");
                    }
                    let source = quarantine_pending_dir().join(&item_id);
                    let canonical_parent = match fs::canonicalize(quarantine_pending_dir()) {
                        Ok(value) => value,
                        Err(_) => return error_response("quarantine_preview_failed", "KidOS quarantine directory is unavailable."),
                    };
                    let canonical_source = match fs::canonicalize(&source) {
                        Ok(value) => value,
                        Err(_) => return error_response("quarantine_preview_failed", "The quarantine item no longer exists."),
                    };
                    if !canonical_source.starts_with(&canonical_parent) || !canonical_source.is_file() {
                        return error_response("quarantine_preview_failed", "KidOS rejected a quarantine preview outside its protected folder.");
                    }
                    match classifier_thumbnail(&canonical_source) {
                        Ok(bytes) => PrivilegedResponse::QuarantinePreview {
                            mime_type: "image/jpeg".into(),
                            data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
                        },
                        Err(message) => error_response("quarantine_preview_failed", &message),
                    }
                }
                Ok(ParentAuthorizationResult::Locked) => error_response("parent_pin_locked", "Parent PIN verification is temporarily locked."),
                Ok(ParentAuthorizationResult::Denied) => error_response("parent_pin_denied", "Parent PIN was not accepted."),
                Err(_) => error_response("parent_pin_error", "KidOS could not verify the parent PIN."),
            }
        }
        PrivilegedRequest::ReviewQuarantine { pin, item_id, action } => {
            match parent_authorization.verify(&pin, now_seconds()) {
                Ok(ParentAuthorizationResult::Authorized) => match review_quarantine_item(&item_id, &action) {
                    Ok(()) => PrivilegedResponse::Ack { message: format!("Quarantine item action '{}' completed.", action) },
                    Err(message) => error_response("quarantine_review_failed", &message),
                },
                Ok(ParentAuthorizationResult::Locked) => error_response("parent_pin_locked", "Parent PIN verification is temporarily locked."),
                Ok(ParentAuthorizationResult::Denied) => error_response("parent_pin_denied", "Parent PIN was not accepted."),
                Err(_) => error_response("parent_pin_error", "KidOS could not verify the parent PIN."),
            }
        }
        PrivilegedRequest::RecoveryStatus => {
            let (lockdown_state, lockdown_reason) = current_platform_state();
            let marker_reason = recovery_reason().or(lockdown_reason);
            PrivilegedResponse::RecoveryStatus {
                guardian_healthy: true,
                classifier_healthy: classifier_healthy(),
                recovery_required: marker_reason.is_some(),
                recovery_reason: marker_reason,
                policy_valid: policy_file_valid(),
                lockdown_state,
            }
        }
        PrivilegedRequest::RunRecovery { pin, action } => {
            match verify_parent(parent_authorization, &pin) {
                Ok(ParentAuthorizationResult::Authorized) => {}
                Ok(ParentAuthorizationResult::Denied) => return error_response("parent_pin_denied", "Parent PIN was not accepted."),
                Ok(ParentAuthorizationResult::Locked) => return error_response("parent_pin_locked", "Parent PIN entry is temporarily locked."),
                Err(error) => return error_response("parent_pin_error", error),
            }

            match action.as_str() {
                "reset_policy_defaults" => match reset_parent_policy_to_defaults(parent_policy) {
                    Ok(()) => PrivilegedResponse::Ack { message: "KidOS parent policy was reset to safe defaults.".into() },
                    Err(message) => error_response("recovery_policy_reset_failed", message),
                },
                "remove_lockdown" => match lockdown_service.remove_lockdown(true) {
                    Ok(()) => {
                        clear_recovery_marker();
                        PrivilegedResponse::Ack { message: "KidOS Windows lockdown was removed for parent recovery.".into() }
                    }
                    Err(error) => error_response("recovery_remove_lockdown_failed", format!("Guardian could not remove lockdown: {error:?}")),
                },
                "clear_recovery_marker" => {
                    if !policy_file_valid() {
                        return error_response("recovery_still_required", "Parent policy is still invalid.");
                    }
                    let (state, reason) = current_platform_state();
                    if state == "restricted_safe_mode" || reason.is_some() {
                        return error_response("recovery_still_required", "Windows lockdown state is still unhealthy.");
                    }
                    clear_recovery_marker();
                    PrivilegedResponse::Ack { message: "KidOS recovery warning was cleared after health checks passed.".into() }
                }
                _ => error_response("invalid_recovery_action", "KidOS rejected an unknown recovery action."),
            }
        }
        PrivilegedRequest::ApplyLockdown { profile } => {
            if let Err(error) = persist_lockdown_profile(&profile) {
                return error_response("recovery_backup_failed", error);
            }
            let profile = match profile_from_ipc(profile) {
                Ok(profile) => profile,
                Err(error) => return error_response("invalid_profile", error),
            };

            match lockdown_service.prepare_and_apply(&profile) {
                Ok(()) => {
                    clear_recovery_marker();
                    PrivilegedResponse::Status { state: "locked".into(), reason: None }
                }
                Err(error) => {
                    mark_recovery_required("assigned-access-apply-failed");
                    let _ = lockdown_service.remove_lockdown(true);
                    error_response(
                        "apply_failed_recovered",
                        format!("Guardian could not apply Windows Assigned Access and removed the partial lockdown: {error:?}"),
                    )
                },
            }
        }
        PrivilegedRequest::ParentUnlock { pin, duration_minutes } => {
            match verify_parent(parent_authorization, &pin) {
                Ok(ParentAuthorizationResult::Authorized) => {}
                Ok(ParentAuthorizationResult::Denied) => return error_response("parent_pin_denied", "Parent PIN was not accepted."),
                Ok(ParentAuthorizationResult::Locked) => return error_response("parent_pin_locked", "Parent PIN entry is temporarily locked."),
                Err(error) => return error_response("parent_pin_error", error),
            }
            match lockdown_service.begin_parent_unlock(true, now_seconds(), duration_minutes) {
                Ok(grant) => PrivilegedResponse::Status {
                    state: "parent_unlocked".into(),
                    reason: Some(format!("Parent maintenance unlock expires at {}.", grant.expires_at)),
                },
                Err(error) => error_response("parent_unlock_failed", format!("Guardian rejected parent unlock: {error:?}")),
            }
        }
        PrivilegedRequest::RemoveLockdown { pin } => {
            match verify_parent(parent_authorization, &pin) {
                Ok(ParentAuthorizationResult::Authorized) => {}
                Ok(ParentAuthorizationResult::Denied) => return error_response("parent_pin_denied", "Parent PIN was not accepted."),
                Ok(ParentAuthorizationResult::Locked) => return error_response("parent_pin_locked", "Parent PIN entry is temporarily locked."),
                Err(error) => return error_response("parent_pin_error", error),
            }
            match lockdown_service.remove_lockdown(true) {
                Ok(()) => PrivilegedResponse::Status { state: "unmanaged".into(), reason: None },
                Err(error) => error_response("remove_lockdown_failed", format!("Guardian could not remove lockdown: {error:?}")),
            }
        }
    }
}

#[cfg(target_os = "windows")]
unsafe fn create_pipe() -> Result<isize, String> {
    // SYSTEM and Administrators get full access; authenticated users can only connect/read/write.
    // Privileged parent-sensitive commands remain denied by the server protocol itself.
    let sddl = wide("D:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;GRGW;;;AU)");
    let mut descriptor = null_mut();
    let ok = ConvertStringSecurityDescriptorToSecurityDescriptorW(
        sddl.as_ptr(),
        1,
        &mut descriptor,
        null_mut(),
    );
    if ok == 0 {
        return Err("Guardian could not create the named-pipe security descriptor.".into());
    }

    let mut attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor,
        bInheritHandle: 0,
    };

    let pipe_name = wide(GUARDIAN_PIPE_NAME);
    let handle = CreateNamedPipeW(
        pipe_name.as_ptr(),
        PIPE_ACCESS_DUPLEX,
        PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
        8,
        MAX_IPC_MESSAGE_BYTES as u32,
        MAX_IPC_MESSAGE_BYTES as u32,
        0,
        &mut attributes,
    );

    let _ = LocalFree(descriptor as isize);

    if handle == INVALID_HANDLE_VALUE {
        Err("Guardian could not create its privileged named pipe.".into())
    } else {
        Ok(handle)
    }
}

#[cfg(target_os = "windows")]
pub fn run_pipe_server() {
    let mut nonce_tracker = PrivilegedNonceTracker::default();
    let mut lockdown_service =
        WindowsLockdownService::new(WindowsAssignedAccessAdapter::default());
    let mut parent_authorization =
        ParentAuthorization::new(WindowsSecretStore::new("KidOSGuardian"), PARENT_PIN_KEY);
    let mut parent_policy = GuardianPolicyStore::default();
    let persisted_policy = load_parent_policy();
    let _ = parent_policy.replace_parent_policy(GuardianActor::ParentAuthorized, persisted_policy);

    loop {
        let pipe = unsafe {
            match create_pipe() {
                Ok(pipe) => pipe,
                Err(error) => {
                    eprintln!("{error}");
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    continue;
                }
            }
        };

        let connected = unsafe {
            ConnectNamedPipe(pipe, null_mut()) != 0 || GetLastError() == ERROR_PIPE_CONNECTED
        };
        if !connected {
            unsafe { CloseHandle(pipe) };
            continue;
        }

        let mut buffer = vec![0u8; MAX_IPC_MESSAGE_BYTES];
        let mut bytes_read = 0u32;
        let read_ok = unsafe {
            ReadFile(
                pipe,
                buffer.as_mut_ptr().cast(),
                buffer.len() as u32,
                &mut bytes_read,
                null_mut(),
            ) != 0
        };

        let response = if read_ok && bytes_read > 0 {
            handle_request(
                &buffer[..bytes_read as usize],
                &mut nonce_tracker,
                &mut lockdown_service,
                &mut parent_authorization,
                &mut parent_policy,
            )
        } else {
            error_response("read_failed", "Guardian could not read the privileged request.")
        };

        let encoded = serde_json::to_vec(&response).unwrap_or_else(|_| {
            br#"{"type":"error","code":"serialization_failed","message":"Guardian could not encode a response."}"#.to_vec()
        });
        let mut bytes_written = 0u32;
        unsafe {
            let _ = WriteFile(
                pipe,
                encoded.as_ptr().cast(),
                encoded.len() as u32,
                &mut bytes_written,
                null_mut(),
            );
            let _ = FlushFileBuffers(pipe);
            let _ = DisconnectNamedPipe(pipe);
            CloseHandle(pipe);
        }
    }
}
