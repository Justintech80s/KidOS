#[cfg(target_os = "windows")]
use guardian_service::{
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
use std::{mem::size_of, ptr::null_mut};
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
        PrivilegedRequest::ParentUnlock { .. } | PrivilegedRequest::RemoveLockdown => error_response(
            "parent_authorization_required",
            "Parent-sensitive lockdown changes are denied at the service boundary until service-side parent authorization is presented.",
        ),
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
