use std::collections::HashSet;

use policy_core::{
    evaluate_navigation, NavigationContext, PolicyDecision, RiskLevel, SiteCategory,
};
use serde::{Deserialize, Serialize};

use crate::GuardianError;

pub const GUARDIAN_PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GuardianRequest {
    GuardianStatus,
    EvaluateNavigation {
        domain: String,
        age: u8,
        parent_blocked: bool,
        parent_allowed: bool,
        category: String,
        risk: String,
        unknown_web_enabled: bool,
    },
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

pub fn evaluate_guardian_request(
    request: &GuardianRequest,
) -> Result<Option<PolicyDecision>, GuardianError> {
    let GuardianRequest::EvaluateNavigation {
        domain,
        age,
        parent_blocked,
        parent_allowed,
        category,
        risk,
        unknown_web_enabled,
    } = request
    else {
        return Ok(None);
    };

    let category = match category.as_str() {
        "unknown" => SiteCategory::Unknown,
        "approved" => SiteCategory::Approved,
        "educational" => SiteCategory::Educational,
        "prohibited" => SiteCategory::Prohibited,
        value => return Err(GuardianError::InvalidPolicyInput(value.to_string())),
    };
    let risk = match risk.as_str() {
        "low" => RiskLevel::Low,
        "normal" => RiskLevel::Normal,
        "high" => RiskLevel::High,
        value => return Err(GuardianError::InvalidPolicyInput(value.to_string())),
    };

    let context = NavigationContext::new(domain.clone(), *age)
        .with_parent_blocked(*parent_blocked)
        .with_parent_allowed(*parent_allowed)
        .with_category(category)
        .with_risk(risk)
        .with_unknown_web_enabled(*unknown_web_enabled);

    Ok(Some(evaluate_navigation(&context)))
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
