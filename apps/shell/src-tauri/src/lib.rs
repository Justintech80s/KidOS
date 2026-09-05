pub mod commands;
#[cfg(target_os = "windows")]
mod guardian_ipc;

use commands::{evaluate_download, evaluate_navigation, get_guardian_status, plan_workspace};
use guardian_service::{GuardianActor, GuardianPolicyStore, ParentPolicyConfig};
use secure_store::{ParentAuthorization, ParentAuthorizationResult, SecretStore};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(target_os = "windows")]
use std::sync::Mutex;
#[cfg(target_os = "windows")]
use guardian_service::windows_lockdown::{
    AccountRole, ApprovedApp, LockdownState as GuardianLockdownState, LockdownStatus as GuardianLockdownStatus,
    LockdownProfile, WindowsAssignedAccessAdapter, WindowsLockdownService,
};
#[cfg(target_os = "windows")]
use secure_store::WindowsSecretStore;

const PARENT_PIN_KEY: &str = "parent-pin";
const KIDOS_ASSIGNED_ACCESS_PROFILE_ID: &str = "{7B62A1F3-8B61-4E6F-9E13-7A4C4A53E9D1}";

pub fn configure_parent_pin_with_store(store: &dyn SecretStore, pin: &str) -> Result<(), String> {
    if !(4..=8).contains(&pin.len()) || !pin.chars().all(|character| character.is_ascii_digit()) {
        return Err("invalid parent PIN".into());
    }
    store.put_secret(PARENT_PIN_KEY, pin).map_err(|_| "unable to protect parent PIN".into())
}

pub fn save_parent_policy_with_authorization<S: SecretStore>(authorization: &mut ParentAuthorization<S>, guardian: &mut GuardianPolicyStore, pin: &str, now_seconds: u64, policy: ParentPolicyConfig) -> Result<(), String> {
    match authorization.verify(pin, now_seconds).map_err(|_| "unable to verify parent PIN".to_string())? {
        ParentAuthorizationResult::Authorized => guardian.replace_parent_policy(GuardianActor::ParentAuthorized, policy).map_err(|error| error.to_string()),
        ParentAuthorizationResult::Denied => Err("parent PIN was not accepted".into()),
        ParentAuthorizationResult::Locked => Err("parent PIN entry is temporarily locked".into()),
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn configure_parent_pin(pin: String, current_pin: Option<String>) -> Result<(), String> {
    guardian_ipc::configure_parent_pin(pin, current_pin)
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn configure_parent_pin(_pin: String, _current_pin: Option<String>) -> Result<(), String> { Err("parent PIN storage is available in the Windows KidOS build".into()) }


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParentVerificationDto {
    authorized: bool,
    locked: bool,
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn verify_parent_pin(pin: String) -> Result<ParentVerificationDto, String> {
    let (authorized, locked) = guardian_ipc::verify_parent_pin(pin)?;
    Ok(ParentVerificationDto { authorized, locked })
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn verify_parent_pin(_pin: String) -> Result<ParentVerificationDto, String> {
    Err("parent PIN verification is available in the Windows KidOS build".into())
}


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParentPolicySaveDto {
    saved: bool,
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn save_parent_policy(
    pin: String,
    policy: ParentPolicyConfig,
    state: tauri::State<'_, LockdownHostState>,
) -> Result<ParentPolicySaveDto, String> {
    guardian_ipc::save_parent_policy(pin, policy.clone())?;
    let mut guardian = state
        .parent_policy
        .lock()
        .map_err(|_| "Guardian policy state is unavailable".to_string())?;
    guardian
        .replace_parent_policy(GuardianActor::ParentAuthorized, policy)
        .map_err(|error| error.to_string())?;
    Ok(ParentPolicySaveDto { saved: true })
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn save_parent_policy(_pin: String, _policy: ParentPolicyConfig) -> Result<ParentPolicySaveDto, String> {
    Err("parent policy saving is available in the Windows KidOS build".into())
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn evaluate_navigation_with_parent_policy(
    url: String,
    state: tauri::State<'_, LockdownHostState>,
) -> Result<String, String> {
    let guardian = state
        .parent_policy
        .lock()
        .map_err(|_| "Guardian policy state is unavailable".to_string())?;
    Ok(commands::evaluate_navigation_with_policy_impl(&url, guardian.current_parent_policy()).to_string())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn evaluate_navigation_with_parent_policy(url: String) -> Result<String, String> {
    Ok(commands::evaluate_navigation_impl(&url).to_string())
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn evaluate_download_with_parent_policy(
    file_name: String,
    mime_type: String,
    state: tauri::State<'_, LockdownHostState>,
) -> Result<String, String> {
    let guardian = state
        .parent_policy
        .lock()
        .map_err(|_| "Guardian policy state is unavailable".to_string())?;
    Ok(commands::evaluate_download_with_policy_impl(
        &file_name,
        &mime_type,
        false,
        false,
        guardian.current_parent_policy(),
    ).to_string())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn evaluate_download_with_parent_policy(file_name: String, mime_type: String) -> Result<String, String> {
    Ok(commands::evaluate_download_impl(&file_name, &mime_type).to_string())
}


#[cfg(target_os = "windows")]
static SAFE_BROWSER_COUNTER: AtomicU64 = AtomicU64::new(1);

#[cfg(target_os = "windows")]
fn is_web_domain(host: &str, domain: &str) -> bool {
    host.eq_ignore_ascii_case(domain)
        || host
            .to_ascii_lowercase()
            .ends_with(&format!(".{}", domain.to_ascii_lowercase()))
}

#[cfg(target_os = "windows")]
fn harden_safe_browser_url(destination: &str) -> Result<tauri::Url, String> {
    let mut url = tauri::Url::parse(destination)
        .map_err(|_| "KidOS could not parse the web address.".to_string())?;

    if !matches!(url.scheme(), "https" | "http") {
        return Err("KidOS Safe Browser allows only http and https destinations.".into());
    }

    let host = url
        .host_str()
        .ok_or_else(|| "KidOS Safe Browser requires a valid web host.".to_string())?
        .to_string();

    if is_web_domain(&host, "google.com") && url.path().starts_with("/search") {
        let mut pairs = url.query_pairs().into_owned().collect::<Vec<_>>();
        pairs.retain(|(key, _)| key != "safe");
        pairs.push(("safe".into(), "active".into()));
        url.query_pairs_mut().clear().extend_pairs(pairs);
    }

    if is_web_domain(&host, "bing.com") && url.path().starts_with("/search") {
        let mut pairs = url.query_pairs().into_owned().collect::<Vec<_>>();
        pairs.retain(|(key, _)| key != "adlt");
        pairs.push(("adlt".into(), "strict".into()));
        url.query_pairs_mut().clear().extend_pairs(pairs);
    }

    Ok(url)
}

#[cfg(target_os = "windows")]
fn safe_browser_navigation_allowed(url: &tauri::Url) -> bool {
    if !matches!(url.scheme(), "https" | "http") {
        return false;
    }

    let Ok(policy) = guardian_ipc::get_parent_policy() else {
        return false;
    };

    if commands::evaluate_navigation_with_policy_impl(url.as_str(), &policy) != "allow" {
        return false;
    }

    let Some(host) = url.host_str() else {
        return false;
    };

    if is_web_domain(host, "google.com")
        && url.path().starts_with("/search")
        && url.query_pairs().find(|(key, value)| key == "safe" && value == "active").is_none()
    {
        return false;
    }

    if is_web_domain(host, "bing.com")
        && url.path().starts_with("/search")
        && url.query_pairs().find(|(key, value)| key == "adlt" && value == "strict").is_none()
    {
        return false;
    }

    true
}


#[cfg(target_os = "windows")]
fn inferred_mime_type(file_name: &str) -> &'static str {
    let ext = std::path::Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match ext.as_str() {
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "zip" => "application/zip",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        "exe" => "application/x-msdownload",
        "msi" => "application/x-msi",
        "bat" => "application/x-bat",
        _ => "application/octet-stream",
    }
}

#[cfg(target_os = "windows")]
fn archive_requires_deep_scan(file_name: &str) -> bool {
    let ext = std::path::Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    matches!(ext.as_str(), "zip" | "rar" | "7z" | "tar" | "gz" | "tgz")
}

#[cfg(target_os = "windows")]
fn sanitize_download_file_name(candidate: &str) -> String {
    let mut result = candidate
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_' | ' ' | '(' | ')') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();

    result = result.trim().trim_matches('.').to_string();
    if result.is_empty() {
        result = "download.bin".into();
    }
    if result.len() > 180 {
        result.truncate(180);
    }
    result
}

#[cfg(target_os = "windows")]
fn safe_download_destination(file_name: &str) -> Result<PathBuf, String> {
    let base = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| "KidOS could not resolve the child Downloads folder.".to_string())?;
    let directory = base.join("Downloads").join("KidOS");
    fs::create_dir_all(&directory)
        .map_err(|_| "KidOS could not create its protected Downloads folder.".to_string())?;
    Ok(directory.join(sanitize_download_file_name(file_name)))
}

#[cfg(target_os = "windows")]
fn browser_download_allowed(
    url: &tauri::Url,
    proposed_destination: &std::path::Path,
) -> Result<PathBuf, String> {
    if !matches!(url.scheme(), "https" | "http") {
        return Err("KidOS blocks downloads from unsupported protocols.".into());
    }

    let candidate = proposed_destination
        .file_name()
        .and_then(|value| value.to_str())
        .or_else(|| {
            url.path_segments()
                .and_then(|mut segments| segments.next_back())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or("download.bin");

    let file_name = sanitize_download_file_name(candidate);
    let mime_type = inferred_mime_type(&file_name).to_string();
    let archive_high_risk = archive_requires_deep_scan(&file_name);

    let decision = guardian_ipc::evaluate_download(
        url.to_string(),
        file_name.clone(),
        mime_type,
        archive_high_risk,
    )?;

    match decision.as_str() {
        "allow" => safe_download_destination(&file_name),
        "require_parent" => Err("Parent approval is required for this download.".into()),
        "block" => Err("KidOS Guardian blocked this download.".into()),
        _ => Err("KidOS Guardian returned an invalid download decision.".into()),
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
async fn open_protected_browser(
    app: tauri::AppHandle,
    url: String,
) -> Result<(), String> {
    let hardened = harden_safe_browser_url(&url)?;
    if !safe_browser_navigation_allowed(&hardened) {
        return Err("KidOS Guardian blocked this web destination.".into());
    }

    let label = format!(
        "kidos-safe-browser-{}-{}",
        now_seconds(),
        SAFE_BROWSER_COUNTER.fetch_add(1, Ordering::Relaxed)
    );

    tauri::WebviewWindowBuilder::new(
        &app,
        label,
        tauri::WebviewUrl::External(hardened),
    )
    .title("KidOS Safe Browser")
    .maximized(true)
    .decorations(false)
    .on_navigation(|next_url| safe_browser_navigation_allowed(next_url))
    .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
    .on_download(|_, event| {
        match event {
            tauri::webview::DownloadEvent::Requested { url, destination } => {
                match browser_download_allowed(url, destination.as_path()) {
                    Ok(safe_destination) => {
                        *destination = safe_destination;
                        true
                    }
                    Err(reason) => {
                        eprintln!("KidOS download denied for {}: {}", url, reason);
                        false
                    }
                }
            }
            tauri::webview::DownloadEvent::Finished { url, path, success } => {
                if success {
                    eprintln!("KidOS protected download completed: {} -> {:?}", url, path);
                } else {
                    eprintln!("KidOS protected download failed: {}", url);
                }
                true
            }
            _ => true,
        }
    })
    .build()
    .map_err(|error| format!("KidOS could not open the protected browser: {error}"))?;

    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
async fn open_protected_browser(_app: tauri::AppHandle, _url: String) -> Result<(), String> {
    Err("KidOS protected browser window is currently available in the Windows build.".into())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LockdownCapabilityDto {
    platform: String,
    supported: bool,
    mechanism: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagedAccountDto {
    id: String,
    display_name: String,
    role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApprovedDesktopAppDto {
    id: String,
    display_name: String,
    executable_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfigureWindowsLockdownRequest {
    account: ManagedAccountDto,
    approved_apps: Vec<ApprovedDesktopAppDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParentUnlockGrantDto {
    granted_at: String,
    expires_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LockdownStatusDto {
    state: String,
    capability: LockdownCapabilityDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    managed_account: Option<ManagedAccountDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_unlock: Option<ParentUnlockGrantDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[cfg(target_os = "windows")]
struct LockdownHostState {
    service: Mutex<WindowsLockdownService<WindowsAssignedAccessAdapter>>,
    managed_account: Mutex<Option<ManagedAccountDto>>,
    parent_authorization: Mutex<ParentAuthorization<WindowsSecretStore>>,
    parent_policy: Mutex<GuardianPolicyStore>,
}


#[cfg(target_os = "windows")]
fn parent_policy_path() -> Result<PathBuf, String> {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "KidOS could not resolve the Windows app-data folder.".to_string())?;
    Ok(base.join("KidOS").join("parent-policy.json"))
}

#[cfg(target_os = "windows")]
fn load_persisted_parent_policy() -> GuardianPolicyStore {
    let mut store = GuardianPolicyStore::default();
    let Ok(path) = parent_policy_path() else { return store; };
    let Ok(contents) = fs::read_to_string(path) else { return store; };
    let Ok(policy) = serde_json::from_str::<ParentPolicyConfig>(&contents) else { return store; };
    let _ = store.replace_parent_policy(GuardianActor::ParentAuthorized, policy);
    store
}

#[cfg(target_os = "windows")]
fn persist_parent_policy(policy: &ParentPolicyConfig) -> Result<(), String> {
    let path = parent_policy_path()?;
    let parent = path.parent().ok_or_else(|| "KidOS policy path is invalid.".to_string())?;
    fs::create_dir_all(parent).map_err(|_| "KidOS could not create its settings folder.".to_string())?;
    let temp = path.with_extension("json.tmp");
    let encoded = serde_json::to_vec_pretty(policy).map_err(|_| "KidOS could not encode parent policy.".to_string())?;
    fs::write(&temp, encoded).map_err(|_| "KidOS could not save parent policy.".to_string())?;
    fs::rename(&temp, &path).map_err(|_| "KidOS could not finalize parent policy.".to_string())?;
    Ok(())
}

#[cfg(target_os = "windows")]
impl LockdownHostState {
    fn new() -> Self {
        Self {
            service: Mutex::new(WindowsLockdownService::new(WindowsAssignedAccessAdapter::default())),
            managed_account: Mutex::new(None),
            parent_authorization: Mutex::new(ParentAuthorization::new(WindowsSecretStore::new("KidOS"), PARENT_PIN_KEY)),
            parent_policy: Mutex::new({
                let mut store = GuardianPolicyStore::default();
                let policy = guardian_ipc::get_parent_policy().unwrap_or_else(|_| load_persisted_parent_policy().current_parent_policy().clone());
                let _ = store.replace_parent_policy(GuardianActor::ParentAuthorized, policy);
                store
            }),
        }
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn iso_from_unix(seconds: u64) -> String {
    // Contract only requires an ISO-like UTC value; JS Date can parse epoch milliseconds
    // once converted by the renderer if needed. Keeping the host dependency-free avoids
    // adding a time library to the privileged boundary.
    format!("{seconds}")
}

#[cfg(target_os = "windows")]
fn capability() -> LockdownCapabilityDto {
    LockdownCapabilityDto {
        platform: "windows".into(),
        supported: true,
        mechanism: "assigned_access".into(),
        reason: None,
    }
}

#[cfg(not(target_os = "windows"))]
fn capability() -> LockdownCapabilityDto {
    LockdownCapabilityDto {
        platform: if cfg!(target_os = "macos") { "macos" } else if cfg!(target_os = "linux") { "linux" } else { "linux" }.into(),
        supported: false,
        mechanism: "unsupported".into(),
        reason: Some("Windows Assigned Access is only available in the Windows KidOS build.".into()),
    }
}

#[cfg(target_os = "windows")]
fn map_state(state: GuardianLockdownState) -> &'static str {
    match state {
        GuardianLockdownState::Unmanaged => "unmanaged",
        GuardianLockdownState::Preparing => "preparing",
        GuardianLockdownState::Locked => "locked",
        GuardianLockdownState::ParentUnlocked => "parent_unlocked",
        GuardianLockdownState::RestrictedSafeMode => "restricted_safe_mode",
    }
}

#[cfg(target_os = "windows")]
fn status_dto(
    status: GuardianLockdownStatus,
    managed_account: Option<ManagedAccountDto>,
    reason: Option<String>,
) -> LockdownStatusDto {
    let now = now_seconds();
    LockdownStatusDto {
        state: map_state(status.state).into(),
        capability: capability(),
        managed_account,
        parent_unlock: status.parent_unlock_expires_at.map(|expires_at| ParentUnlockGrantDto {
            granted_at: iso_from_unix(now),
            expires_at: iso_from_unix(expires_at),
        }),
        reason,
    }
}


#[cfg(target_os = "windows")]
fn ipc_status_dto(
    state: String,
    managed_account: Option<ManagedAccountDto>,
    reason: Option<String>,
) -> LockdownStatusDto {
    LockdownStatusDto {
        state,
        capability: capability(),
        managed_account,
        parent_unlock: None,
        reason,
    }
}

#[cfg(target_os = "windows")]
fn account_role(value: &str) -> AccountRole {
    match value {
        "standard" => AccountRole::Standard,
        "administrator" => AccountRole::Administrator,
        _ => AccountRole::Unknown,
    }
}

#[cfg(target_os = "windows")]
fn build_profile(request: &ConfigureWindowsLockdownRequest) -> Result<LockdownProfile, String> {
    if request.account.role != "standard" {
        return Err("Windows Lockdown Mode requires a standard child account.".into());
    }

    let current_exe = std::env::current_exe()
        .map_err(|_| "KidOS could not resolve its installed executable path.".to_string())?
        .to_string_lossy()
        .to_string();

    let mut apps = Vec::with_capacity(request.approved_apps.len().max(1));
    let mut has_kidos = false;
    for app in &request.approved_apps {
        let path = if app.id.eq_ignore_ascii_case("kidos") {
            has_kidos = true;
            current_exe.clone()
        } else {
            app.executable_path.clone()
        };
        apps.push(ApprovedApp {
            id: app.id.clone(),
            display_name: app.display_name.clone(),
            executable_path: path,
        });
    }
    if !has_kidos {
        apps.push(ApprovedApp {
            id: "kidos".into(),
            display_name: "KidOS".into(),
            executable_path: current_exe,
        });
    }

    Ok(LockdownProfile {
        profile_id: KIDOS_ASSIGNED_ACCESS_PROFILE_ID.into(),
        account: request.account.id.clone(),
        account_role: account_role(&request.account.role),
        apps,
    })
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn lockdown_status(state: tauri::State<'_, LockdownHostState>) -> Result<LockdownStatusDto, String> {
    let managed = state
        .managed_account
        .lock()
        .map_err(|_| "Guardian account state is unavailable".to_string())?
        .clone();

    match guardian_ipc::status() {
        Ok((ipc_state, reason)) => Ok(ipc_status_dto(ipc_state, managed, reason)),
        Err(error) => Ok(ipc_status_dto(
            "restricted_safe_mode".into(),
            managed,
            Some(format!("Privileged Guardian service unavailable: {error}")),
        )),
    }
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn lockdown_status() -> Result<LockdownStatusDto, String> {
    Ok(LockdownStatusDto {
        state: "unmanaged".into(),
        capability: capability(),
        managed_account: None,
        parent_unlock: None,
        reason: Some("Windows Lockdown Mode is unavailable on this platform.".into()),
    })
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn configure_windows_lockdown(
    request: ConfigureWindowsLockdownRequest,
    state: tauri::State<'_, LockdownHostState>,
) -> Result<LockdownStatusDto, String> {
    let profile = build_profile(&request)?;

    let (ipc_state, reason) = guardian_ipc::apply(&profile)
        .map_err(|error| format!("Privileged Guardian service rejected lockdown request: {error}"))?;

    *state
        .managed_account
        .lock()
        .map_err(|_| "Guardian account state is unavailable".to_string())? = Some(request.account.clone());

    Ok(ipc_status_dto(ipc_state, Some(request.account), reason))
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn configure_windows_lockdown(_request: ConfigureWindowsLockdownRequest) -> Result<LockdownStatusDto, String> {
    lockdown_status()
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn request_parent_maintenance_unlock(
    pin: String,
    duration_minutes: u64,
) -> Result<ParentUnlockGrantDto, String> {
    let now = now_seconds();
    let (_state, reason) = guardian_ipc::parent_unlock(pin, duration_minutes)?;
    let expires_at = now.saturating_add(duration_minutes.saturating_mul(60));
    Ok(ParentUnlockGrantDto {
        granted_at: iso_from_unix(now),
        expires_at: iso_from_unix(expires_at),
    })
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn request_parent_maintenance_unlock(_pin: String, _duration_minutes: u64) -> Result<ParentUnlockGrantDto, String> {
    Err("Windows Lockdown Mode is unavailable on this platform.".into())
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn remove_windows_lockdown(pin: String, state: tauri::State<'_, LockdownHostState>) -> Result<LockdownStatusDto, String> {
    let (ipc_state, reason) = guardian_ipc::remove_lockdown(pin)?;
    *state
        .managed_account
        .lock()
        .map_err(|_| "Guardian account state is unavailable".to_string())? = None;
    Ok(ipc_status_dto(ipc_state, None, reason))
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn remove_windows_lockdown(_pin: String) -> Result<LockdownStatusDto, String> {
    lockdown_status()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    #[cfg(target_os = "windows")]
    let builder = builder.manage(LockdownHostState::new());

    builder
        .invoke_handler(tauri::generate_handler![
            configure_parent_pin,
            verify_parent_pin,
            save_parent_policy,
            evaluate_navigation_with_parent_policy,
            evaluate_download_with_parent_policy,
            open_protected_browser,
            plan_workspace,
            evaluate_navigation,
            evaluate_download,
            get_guardian_status,
            lockdown_status,
            configure_windows_lockdown,
            request_parent_maintenance_unlock,
            remove_windows_lockdown
        ])
        .run(tauri::generate_context!())
        .expect("error while running KidOS");
}
