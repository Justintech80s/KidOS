use crate::Capability;
use std::{collections::BTreeSet, error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcessRole {
    Guardian,
    Shell,
    SafeBrowser,
    MediaClassifier,
    ApprovedApp,
    ParentControl,
    Compositor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServiceRoute {
    pub from: ProcessRole,
    pub to: ProcessRole,
    pub required_capability: Capability,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SessionTopology {
    routes: BTreeSet<ServiceRoute>,
}

impl SessionTopology {
    pub fn child_default() -> Self {
        let routes = [
            ServiceRoute {
                from: ProcessRole::Shell,
                to: ProcessRole::Guardian,
                required_capability: Capability::LaunchApprovedApp,
            },
            ServiceRoute {
                from: ProcessRole::SafeBrowser,
                to: ProcessRole::Guardian,
                required_capability: Capability::SafeBrowser,
            },
            ServiceRoute {
                from: ProcessRole::Guardian,
                to: ProcessRole::MediaClassifier,
                required_capability: Capability::ReadChildMedia,
            },
            ServiceRoute {
                from: ProcessRole::ParentControl,
                to: ProcessRole::Guardian,
                required_capability: Capability::ParentSettings,
            },
        ];
        Self {
            routes: routes.into_iter().collect(),
        }
    }

    pub fn authorize_route(
        &self,
        from: ProcessRole,
        to: ProcessRole,
        capability: Capability,
    ) -> Result<(), TopologyError> {
        let route = ServiceRoute {
            from,
            to,
            required_capability: capability,
        };
        if self.routes.contains(&route) {
            Ok(())
        } else {
            Err(TopologyError(route))
        }
    }

    pub fn routes(&self) -> impl Iterator<Item = ServiceRoute> + '_ {
        self.routes.iter().copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopologyError(pub ServiceRoute);

impl fmt::Display for TopologyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "route {:?} -> {:?} with {:?} is not declared",
            self.0.from, self.0.to, self.0.required_capability
        )
    }
}

impl Error for TopologyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undeclared_process_routes_are_denied() {
        let topology = SessionTopology::child_default();
        assert!(topology
            .authorize_route(
                ProcessRole::SafeBrowser,
                ProcessRole::Guardian,
                Capability::SafeBrowser,
            )
            .is_ok());
        assert!(topology
            .authorize_route(
                ProcessRole::ApprovedApp,
                ProcessRole::Guardian,
                Capability::GuardianControl,
            )
            .is_err());
    }
}
