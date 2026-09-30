use crate::{Capability, CapabilitySet, MissingCapability};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AppIdentity(String);

impl AppIdentity {
    pub fn new(id: impl Into<String>) -> Result<Self, &'static str> {
        let id = id.into();
        if id.trim().is_empty() || id.len() > 128 {
            return Err("app identity must be between 1 and 128 characters");
        }
        if !id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_'))
        {
            return Err("app identity contains unsupported characters");
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxProfile {
    app: AppIdentity,
    capabilities: CapabilitySet,
}

impl SandboxProfile {
    pub fn new(app: AppIdentity, capabilities: CapabilitySet) -> Self {
        Self { app, capabilities }
    }

    pub fn app(&self) -> &AppIdentity {
        &self.app
    }

    pub fn allows(&self, capability: Capability) -> bool {
        self.capabilities.allows(capability)
    }

    pub fn authorize(&self, capability: Capability) -> Result<(), MissingCapability> {
        self.capabilities.require(capability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_identity_rejects_unsafe_names() {
        assert!(AppIdentity::new("com.kidos.safe-browser").is_ok());
        assert!(AppIdentity::new("../guardian").is_err());
        assert!(AppIdentity::new("").is_err());
    }

    #[test]
    fn sandbox_profile_is_least_privilege() {
        let profile = SandboxProfile::new(
            AppIdentity::new("com.example.creator").unwrap(),
            CapabilitySet::from_capabilities([
                Capability::Camera,
                Capability::ReadChildMedia,
                Capability::Network,
            ]),
        );
        assert!(profile.authorize(Capability::Camera).is_ok());
        assert!(profile.authorize(Capability::GuardianControl).is_err());
        assert!(profile.authorize(Capability::SystemSettings).is_err());
    }
}
