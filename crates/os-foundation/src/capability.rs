use std::{collections::BTreeSet, error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    Camera,
    Microphone,
    Network,
    ReadChildMedia,
    WriteChildMedia,
    LaunchApprovedApp,
    SafeBrowser,
    ParentSettings,
    GuardianControl,
    SystemSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CapabilitySet {
    granted: BTreeSet<Capability>,
}

impl CapabilitySet {
    pub fn deny_all() -> Self {
        Self::default()
    }

    pub fn from_capabilities(capabilities: impl IntoIterator<Item = Capability>) -> Self {
        Self {
            granted: capabilities.into_iter().collect(),
        }
    }

    pub fn allows(&self, capability: Capability) -> bool {
        self.granted.contains(&capability)
    }

    pub fn require(&self, capability: Capability) -> Result<(), MissingCapability> {
        self.allows(capability)
            .then_some(())
            .ok_or(MissingCapability(capability))
    }

    pub fn iter(&self) -> impl Iterator<Item = Capability> + '_ {
        self.granted.iter().copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingCapability(pub Capability);

impl fmt::Display for MissingCapability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "required capability {:?} was not granted", self.0)
    }
}

impl Error for MissingCapability {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_sets_deny_by_default() {
        let capabilities = CapabilitySet::deny_all();
        assert!(!capabilities.allows(Capability::Network));
        assert!(capabilities.require(Capability::Network).is_err());
    }

    #[test]
    fn only_explicit_capabilities_are_allowed() {
        let capabilities = CapabilitySet::from_capabilities([
            Capability::Network,
            Capability::ReadChildMedia,
        ]);
        assert!(capabilities.allows(Capability::Network));
        assert!(capabilities.allows(Capability::ReadChildMedia));
        assert!(!capabilities.allows(Capability::SystemSettings));
    }
}
