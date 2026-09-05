#[cfg(target_os = "windows")]
use guardian_service::{
    privileged_ipc::{
        IpcAccountRole, IpcApprovedApp, IpcLockdownProfile, PrivilegedRequest,
        PrivilegedRequestEnvelope, PrivilegedResponse, GUARDIAN_PIPE_NAME,
        PRIVILEGED_PROTOCOL_VERSION,
    },
    windows_lockdown::{AccountRole, LockdownProfile},
    ParentPolicyConfig,
};
#[cfg(target_os = "windows")]
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(target_os = "windows")]
static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[cfg(target_os = "windows")]
fn request_identity() -> (String, String) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    let counter = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    (
        format!("shell-{pid}-{now}"),
        format!("{pid}-{now}-{counter}"),
    )
}

#[cfg(target_os = "windows")]
fn send(request: PrivilegedRequest) -> Result<PrivilegedResponse, String> {
    let (session_id, nonce) = request_identity();
    let envelope = PrivilegedRequestEnvelope {
        version: PRIVILEGED_PROTOCOL_VERSION,
        session_id,
        nonce,
        request,
    };

    let encoded = serde_json::to_vec(&envelope)
        .map_err(|_| "KidOS could not encode Guardian IPC request.".to_string())?;

    let mut pipe = OpenOptions::new()
        .read(true)
        .write(true)
        .open(GUARDIAN_PIPE_NAME)
        .map_err(|error| format!("KidOS Guardian service is unavailable: {error}"))?;

    pipe.write_all(&encoded)
        .map_err(|_| "KidOS could not send a request to Guardian.".to_string())?;
    pipe.flush()
        .map_err(|_| "KidOS could not flush the Guardian request.".to_string())?;

    let mut response = Vec::new();
    pipe.read_to_end(&mut response)
        .map_err(|_| "KidOS could not read the Guardian response.".to_string())?;

    serde_json::from_slice(&response)
        .map_err(|_| "KidOS received an invalid Guardian response.".to_string())
}

#[cfg(target_os = "windows")]
fn ipc_profile(profile: &LockdownProfile) -> IpcLockdownProfile {
    IpcLockdownProfile {
        profile_id: profile.profile_id.clone(),
        account: profile.account.clone(),
        account_role: match profile.account_role {
            AccountRole::Standard => IpcAccountRole::Standard,
            AccountRole::Administrator => IpcAccountRole::Administrator,
            AccountRole::Unknown => IpcAccountRole::Unknown,
        },
        apps: profile
            .apps
            .iter()
            .map(|app| IpcApprovedApp {
                id: app.id.clone(),
                display_name: app.display_name.clone(),
                executable_path: app.executable_path.clone(),
            })
            .collect(),
    }
}

#[cfg(target_os = "windows")]
pub fn status() -> Result<(String, Option<String>), String> {
    match send(PrivilegedRequest::Status)? {
        PrivilegedResponse::Status { state, reason } => Ok((state, reason)),
        PrivilegedResponse::Error { code, message } => Err(format!("{code}: {message}")),
    }
}

#[cfg(target_os = "windows")]
pub fn apply(profile: &LockdownProfile) -> Result<(String, Option<String>), String> {
    match send(PrivilegedRequest::ApplyLockdown {
        profile: ipc_profile(profile),
    })? {
        PrivilegedResponse::Status { state, reason } => Ok((state, reason)),
        PrivilegedResponse::Error { code, message } => Err(format!("{code}: {message}")),
    }
}


#[cfg(target_os = "windows")]
pub fn configure_parent_pin(new_pin: String, current_pin: Option<String>) -> Result<(), String> {
    match send(PrivilegedRequest::ConfigureParentPin { new_pin, current_pin })? {
        PrivilegedResponse::Ack { .. } => Ok(()),
        PrivilegedResponse::Error { code, message } => Err(format!("{code}: {message}")),
        _ => Err("Guardian returned an unexpected PIN configuration response.".into()),
    }
}

#[cfg(target_os = "windows")]
pub fn verify_parent_pin(pin: String) -> Result<(bool, bool), String> {
    match send(PrivilegedRequest::VerifyParentPin { pin })? {
        PrivilegedResponse::ParentVerification { authorized, locked } => Ok((authorized, locked)),
        PrivilegedResponse::Error { code, message } => Err(format!("{code}: {message}")),
        _ => Err("Guardian returned an unexpected parent verification response.".into()),
    }
}

#[cfg(target_os = "windows")]
pub fn save_parent_policy(pin: String, policy: ParentPolicyConfig) -> Result<(), String> {
    match send(PrivilegedRequest::SaveParentPolicy { pin, policy })? {
        PrivilegedResponse::Ack { .. } => Ok(()),
        PrivilegedResponse::Error { code, message } => Err(format!("{code}: {message}")),
        _ => Err("Guardian returned an unexpected policy response.".into()),
    }
}

#[cfg(target_os = "windows")]
pub fn get_parent_policy() -> Result<ParentPolicyConfig, String> {
    match send(PrivilegedRequest::GetParentPolicy)? {
        PrivilegedResponse::ParentPolicy { policy } => Ok(policy),
        PrivilegedResponse::Error { code, message } => Err(format!("{code}: {message}")),
        _ => Err("Guardian returned an unexpected policy response.".into()),
    }
}

#[cfg(target_os = "windows")]
pub fn parent_unlock(pin: String, duration_minutes: u64) -> Result<(String, Option<String>), String> {
    match send(PrivilegedRequest::ParentUnlock { pin, duration_minutes })? {
        PrivilegedResponse::Status { state, reason } => Ok((state, reason)),
        PrivilegedResponse::Error { code, message } => Err(format!("{code}: {message}")),
        _ => Err("Guardian returned an unexpected unlock response.".into()),
    }
}

#[cfg(target_os = "windows")]
pub fn remove_lockdown(pin: String) -> Result<(String, Option<String>), String> {
    match send(PrivilegedRequest::RemoveLockdown { pin })? {
        PrivilegedResponse::Status { state, reason } => Ok((state, reason)),
        PrivilegedResponse::Error { code, message } => Err(format!("{code}: {message}")),
        _ => Err("Guardian returned an unexpected lockdown removal response.".into()),
    }
}


#[cfg(target_os = "windows")]
pub fn evaluate_download(
    url: String,
    file_name: String,
    mime_type: String,
    archive_contains_high_risk: bool,
) -> Result<String, String> {
    match send(PrivilegedRequest::EvaluateDownload {
        url,
        file_name,
        mime_type,
        archive_contains_high_risk,
    })? {
        PrivilegedResponse::PolicyDecision { decision } => Ok(decision),
        PrivilegedResponse::Error { code, message } => Err(format!("{code}: {message}")),
        _ => Err("Guardian returned an unexpected download-policy response.".into()),
    }
}


#[cfg(target_os = "windows")]
pub fn evaluate_media(
    file_name: String,
    category: String,
    risk: String,
    high_confidence: bool,
    classifier_available: bool,
) -> Result<String, String> {
    match send(PrivilegedRequest::EvaluateMedia {
        file_name,
        category,
        risk,
        high_confidence,
        classifier_available,
    })? {
        PrivilegedResponse::PolicyDecision { decision } => Ok(decision),
        PrivilegedResponse::Error { code, message } => Err(format!("{code}: {message}")),
        _ => Err("Guardian returned an unexpected media-policy response.".into()),
    }
}
