mod ipc;
mod policy_store;
mod service_state;

use std::{error::Error, fmt};

pub use ipc::{
    decode_request, validate_request_for_actor, GuardianActor, GuardianRequest, NonceTracker,
    RequestEnvelope, GUARDIAN_PROTOCOL_VERSION,
};
pub use policy_store::PolicySnapshot;
pub use service_state::{load_service_state, GuardianMode, GuardianState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardianError {
    MalformedRequest(String),
    UnsupportedVersion(u16),
    DuplicateNonce,
    UnauthorizedRequest,
}

impl fmt::Display for GuardianError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedRequest(message) => write!(formatter, "malformed Guardian request: {message}"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported Guardian protocol version: {version}")
            }
            Self::DuplicateNonce => write!(formatter, "duplicate Guardian request nonce"),
            Self::UnauthorizedRequest => write!(formatter, "Guardian request is not authorized"),
        }
    }
}

impl Error for GuardianError {}
