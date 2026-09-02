use serde::{Deserialize, Serialize};

use crate::{GuardianActor, GuardianError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParentDownloadMode {
    BlockHighRisk,
    RequireParentHighRisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SocialAccessMode {
    Blocked,
    Allowed,
    TimeLimited,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SocialAccessRule {
    pub service: String,
    pub mode: SocialAccessMode,
    pub start_minutes: Option<u16>,
    pub end_minutes: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParentPolicyConfig {
    pub child_age: u8,
    pub allow_domains: Vec<String>,
    pub block_domains: Vec<String>,
    pub teen_unknown_web_enabled: bool,
    pub social_access: Vec<SocialAccessRule>,
    pub download_mode: ParentDownloadMode,
}

impl Default for ParentPolicyConfig {
    fn default() -> Self {
        Self {
            child_age: 10,
            allow_domains: Vec::new(),
            block_domains: Vec::new(),
            teen_unknown_web_enabled: false,
            social_access: Vec::new(),
            download_mode: ParentDownloadMode::RequireParentHighRisk,
        }
    }
}

fn valid_domain(domain: &str) -> bool {
    let domain = domain.trim();
    !domain.is_empty()
        && domain.len() <= 253
        && !domain.contains('/')
        && !domain.contains(':')
        && domain.contains('.')
}

fn validate_policy(policy: &ParentPolicyConfig) -> Result<(), GuardianError> {
    if !(3..=17).contains(&policy.child_age) {
        return Err(GuardianError::InvalidPolicyInput(
            "child age must be from 3 through 17".into(),
        ));
    }

    if policy.child_age < 13 && policy.teen_unknown_web_enabled {
        return Err(GuardianError::InvalidPolicyInput(
            "unknown-web access is available only for teen profiles".into(),
        ));
    }

    if policy.allow_domains.iter().chain(&policy.block_domains).any(|domain| !valid_domain(domain)) {
        return Err(GuardianError::InvalidPolicyInput(
            "domain rules must contain valid host names".into(),
        ));
    }

    for rule in &policy.social_access {
        if rule.service.trim().is_empty() {
            return Err(GuardianError::InvalidPolicyInput(
                "social service name cannot be empty".into(),
            ));
        }

        match rule.mode {
            SocialAccessMode::TimeLimited => {
                let (Some(start), Some(end)) = (rule.start_minutes, rule.end_minutes) else {
                    return Err(GuardianError::InvalidPolicyInput(
                        "time-limited social access requires a start and end time".into(),
                    ));
                };
                if start >= end || end > 1440 {
                    return Err(GuardianError::InvalidPolicyInput(
                        "social access start time must be before the end time".into(),
                    ));
                }
            }
            SocialAccessMode::Blocked | SocialAccessMode::Allowed => {
                if rule.start_minutes.is_some() || rule.end_minutes.is_some() {
                    return Err(GuardianError::InvalidPolicyInput(
                        "social time windows are allowed only in time-limited mode".into(),
                    ));
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct GuardianPolicyStore {
    parent_policy: ParentPolicyConfig,
}

impl GuardianPolicyStore {
    pub fn current_parent_policy(&self) -> &ParentPolicyConfig {
        &self.parent_policy
    }

    pub fn replace_parent_policy(
        &mut self,
        actor: GuardianActor,
        policy: ParentPolicyConfig,
    ) -> Result<(), GuardianError> {
        if actor != GuardianActor::ParentAuthorized {
            return Err(GuardianError::UnauthorizedRequest);
        }

        validate_policy(&policy)?;
        self.parent_policy = policy;
        Ok(())
    }
}
