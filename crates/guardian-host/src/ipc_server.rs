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
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, ERROR_PIPE_CONNECTED, INVALID_HANDLE_VALUE},
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
    let Ok(contents) = fs::read_to_string(path) else { return ParentPolicyConfig::default(); };
    serde_json::from_str(&contents).unwrap_or_default()
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
fn profile_from_ipc(profile: guardian_service::privileged_ipc::IpcLockdownProfile) -> Result<LockdownProfile, String> {
    if profile.account.trim().is_empty() {
        return Err("child account cannot be empty".into());
    }
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
        apps.push(ApprovedApp {
            id: app.id,
            display_name: app.display_name,
            executable_path: app.executable_path,
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
        PrivilegedRequest::ApplyLockdown { profile } => {
            let profile = match profile_from_ipc(profile) {
                Ok(profile) => profile,
                Err(error) => return error_response("invalid_profile", error),
            };

            match lockdown_service.prepare_and_apply(&profile) {
                Ok(()) => PrivilegedResponse::Status { state: "locked".into(), reason: None },
                Err(error) => error_response(
                    "apply_failed",
                    format!("Guardian could not apply Windows Assigned Access: {error:?}"),
                ),
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
