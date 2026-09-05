use serde::{Deserialize, Serialize};
use crate::ParentPolicyConfig;
use std::collections::HashSet;

pub const GUARDIAN_PIPE_NAME: &str = r"\\.\pipe\KidOSGuardian.v1";
pub const PRIVILEGED_PROTOCOL_VERSION: u16 = 1;
pub const MAX_IPC_MESSAGE_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpcAccountRole {
    Standard,
    Administrator,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IpcApprovedApp {
    pub id: String,
    pub display_name: String,
    pub executable_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IpcLockdownProfile {
    pub profile_id: String,
    pub account: String,
    pub account_role: IpcAccountRole,
    pub apps: Vec<IpcApprovedApp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum PrivilegedRequest {
    Status,
    ConfigureParentPin { new_pin: String, current_pin: Option<String> },
    VerifyParentPin { pin: String },
    SaveParentPolicy { pin: String, policy: ParentPolicyConfig },
    GetParentPolicy,
    ApplyLockdown { profile: IpcLockdownProfile },
    ParentUnlock { pin: String, duration_minutes: u64 },
    RemoveLockdown { pin: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrivilegedRequestEnvelope {
    pub version: u16,
    pub session_id: String,
    pub nonce: String,
    pub request: PrivilegedRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PrivilegedResponse {
    Status { state: String, reason: Option<String> },
    ParentVerification { authorized: bool, locked: bool },
    ParentPolicy { policy: ParentPolicyConfig },
    Ack { message: String },
    Error { code: String, message: String },
}

#[derive(Debug, Default)]
pub struct PrivilegedNonceTracker {
    seen: HashSet<(String, String)>,
}

impl PrivilegedNonceTracker {
    pub fn accept(&mut self, envelope: &PrivilegedRequestEnvelope) -> Result<(), &'static str> {
        if envelope.version != PRIVILEGED_PROTOCOL_VERSION {
            return Err("unsupported_protocol");
        }
        if envelope.session_id.trim().is_empty() || envelope.nonce.trim().is_empty() {
            return Err("missing_request_identity");
        }
        if envelope.session_id.len() > 128 || envelope.nonce.len() > 128 {
            return Err("request_identity_too_long");
        }
        if !self.seen.insert((envelope.session_id.clone(), envelope.nonce.clone())) {
            return Err("replayed_request");
        }
        if self.seen.len() > 4096 {
            self.seen.clear();
            self.seen.insert((envelope.session_id.clone(), envelope.nonce.clone()));
        }
        Ok(())
    }
}

pub fn decode_privileged_request(bytes: &[u8]) -> Result<PrivilegedRequestEnvelope, String> {
    if bytes.is_empty() || bytes.len() > MAX_IPC_MESSAGE_BYTES {
        return Err("invalid IPC message length".into());
    }
    serde_json::from_slice(bytes).map_err(|error| format!("invalid Guardian IPC request: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_replayed_privileged_requests() {
        let request = PrivilegedRequestEnvelope {
            version: PRIVILEGED_PROTOCOL_VERSION,
            session_id: "session".into(),
            nonce: "one".into(),
            request: PrivilegedRequest::Status,
        };
        let mut tracker = PrivilegedNonceTracker::default();
        assert!(tracker.accept(&request).is_ok());
        assert_eq!(tracker.accept(&request), Err("replayed_request"));
    }
}
