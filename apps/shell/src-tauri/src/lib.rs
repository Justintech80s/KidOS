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
