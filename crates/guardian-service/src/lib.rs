mod ipc;
mod parent_policy;
mod policy_store;
pub mod privileged_ipc;
mod safety_events;
mod service_state;
pub mod windows_lockdown;

use std::{error::Error, fmt};

pub use ipc::{
    decode_request, evaluate_guardian_request, validate_request_for_actor, GuardianActor,
    GuardianRequest, NonceTracker, RequestEnvelope, GUARDIAN_PROTOCOL_VERSION,
};
pub use parent_policy::{
    GuardianPolicyStore, ParentDownloadMode, ParentPolicyConfig, SocialAccessMode, SocialAccessRule,
};
pub use policy_store::PolicySnapshot;
pub use safety_events::{SafetyEvent, SafetyEventError, SafetyEventStore, SafetyEventSummary};
pub use service_state::{load_service_state, GuardianMode, GuardianState};
pub use windows_lockdown::{
    AccountRole, ApprovedApp, InMemoryWindowsLockdownAdapter, LockdownAdapterError,
    LockdownProfile, LockdownServiceError, LockdownState, LockdownStatus, ParentUnlockGrant,
    WindowsLockdownService,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardianError {
    MalformedRequest(String),
    UnsupportedVersion(u16),
    InvalidPolicyInput(String),
    DuplicateNonce,
    UnauthorizedRequest,
}

impl fmt::Display for GuardianError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedRequest(message) => write!(formatter, "malformed Guardian request: {message}"),
            Self::UnsupportedVersion(version) => write!(formatter, "unsupported Guardian protocol version: {version}"),
            Self::InvalidPolicyInput(value) => write!(formatter, "invalid Guardian policy input: {value}"),
            Self::DuplicateNonce => write!(formatter, "duplicate Guardian request nonce"),
            Self::UnauthorizedRequest => write!(formatter, "Guardian request is not authorized"),
        }
    }
}

impl Error for GuardianError {}
