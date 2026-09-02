use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::GuardianError;

pub const GUARDIAN_PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GuardianRequest {
    GuardianStatus,
    ReplacePolicy { policy_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestEnvelope {
    pub version: u16,
    pub session_id: String,
    pub nonce: String,
    pub request: GuardianRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardianActor {
    Child,
    ParentAuthorized,
}

pub fn decode_request(bytes: &[u8]) -> Result<RequestEnvelope, GuardianError> {
    let envelope: RequestEnvelope = serde_json::from_slice(bytes)
        .map_err(|error| GuardianError::MalformedRequest(error.to_string()))?;

    if envelope.version != GUARDIAN_PROTOCOL_VERSION {
        return Err(GuardianError::UnsupportedVersion(envelope.version));
    }
    if envelope.session_id.trim().is_empty() || envelope.nonce.trim().is_empty() {
        return Err(GuardianError::MalformedRequest(
            "session_id and nonce must be non-empty".into(),
        ));
    }

    Ok(envelope)
}

pub fn validate_request_for_actor(
    envelope: &RequestEnvelope,
    actor: GuardianActor,
) -> Result<(), GuardianError> {
    match (&envelope.request, actor) {
        (GuardianRequest::ReplacePolicy { .. }, GuardianActor::Child) => {
            Err(GuardianError::UnauthorizedRequest)
        }
        _ => Ok(()),
    }
}

#[derive(Debug, Default)]
pub struct NonceTracker {
    seen: HashSet<(String, String)>,
}

impl NonceTracker {
    pub fn accept(&mut self, envelope: &RequestEnvelope) -> Result<(), GuardianError> {
        let key = (envelope.session_id.clone(), envelope.nonce.clone());
        if !self.seen.insert(key) {
            return Err(GuardianError::DuplicateNonce);
        }
        Ok(())
    }
}
