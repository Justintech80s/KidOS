mod ipc;
pub mod linux_lockdown;
mod parent_policy;
pub mod platform;
mod policy_store;
pub mod privileged_ipc;
mod safety_events;
mod service_state;
mod system_guard;
pub mod windows_lockdown;

use std::{error::Error, fmt};

pub use ipc::{
    decode_request, evaluate_guardian_request, validate_request_for_actor, GuardianActor,
    GuardianRequest, NonceTracker, RequestEnvelope, GUARDIAN_PROTOCOL_VERSION,
};
pub use os_foundation::{
    AppIdentity, BootError, BootPhase, BootSequence, Capability, CapabilitySet, CoreHealth,
    MissingCapability, ProcessRole, RecoveryEvent, RecoveryMachine, RecoveryState, SandboxProfile,
    ServiceRoute, SessionTopology, SystemMode, TopologyError,
};
pub use linux_lockdown::{
    build_linux_platform_config, LinuxLockdownConfigError, LinuxLockdownProfile,
    LinuxPlatformAdapter, LinuxPlatformBackend, ProductionLinuxPlatformAdapter,
    SystemLinuxBackend, KIDOS_CHILD_SERVICE, KIDOS_CHILD_SERVICE_PATH, KIDOS_CHILD_TARGET,
    KIDOS_CHILD_TARGET_PATH, KIDOS_LINUX_CONFIG_PATH,
};
pub use parent_policy::{
    GuardianPolicyStore, ParentDownloadMode, ParentPolicyConfig, SocialAccessMode, SocialAccessRule,
};
pub use platform::{
    InMemoryPlatformAdapter, PlatformAdapterError, PlatformInspection, PlatformKind,
    PlatformLockdownAdapter, PlatformLockdownConfig, PlatformLockdownService,
    PlatformLockdownServiceError, PlatformLockdownState, PlatformLockdownStatus,
    PlatformUnlockGrant,
};
pub use policy_store::PolicySnapshot;
pub use safety_events::{SafetyEvent, SafetyEventError, SafetyEventStore, SafetyEventSummary};
pub use service_state::{load_service_state, GuardianMode, GuardianState};
pub use system_guard::guardian_mode_from_core_health;
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
