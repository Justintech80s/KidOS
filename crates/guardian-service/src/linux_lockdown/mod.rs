use crate::platform::{
    PlatformAdapterError, PlatformInspection, PlatformKind, PlatformLockdownAdapter,
    PlatformLockdownConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const KIDOS_LINUX_CONFIG_PATH: &str = "/var/lib/kidos/lockdown.json";
pub const KIDOS_CHILD_SERVICE_PATH: &str = "/etc/systemd/system/kidos-child-session.service";
pub const KIDOS_CHILD_TARGET_PATH: &str = "/etc/systemd/system/kidos-child.target";
pub const KIDOS_CHILD_SERVICE: &str = "kidos-child-session.service";
pub const KIDOS_CHILD_TARGET: &str = "kidos-child.target";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinuxLockdownProfile {
    pub child_user: String,
    pub shell_path: String,
    pub guardian_unit: String,
    #[serde(default)]
    pub approved_apps: Vec<String>,
}

impl LinuxLockdownProfile {
    pub fn validate(&self) -> Result<(), LinuxLockdownConfigError> {
        if !valid_linux_user(&self.child_user) {
            return Err(LinuxLockdownConfigError::InvalidChildUser);
        }
        if !valid_absolute_path(&self.shell_path) {
            return Err(LinuxLockdownConfigError::InvalidShellPath);
        }
        if !valid_systemd_service_name(&self.guardian_unit) {
            return Err(LinuxLockdownConfigError::InvalidGuardianUnit);
        }

        let mut seen = BTreeSet::new();
        for path in &self.approved_apps {
            if !valid_absolute_path(path) {
                return Err(LinuxLockdownConfigError::InvalidApprovedApp(path.clone()));
            }
            if !seen.insert(path) {
                return Err(LinuxLockdownConfigError::DuplicateApprovedApp(path.clone()));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinuxLockdownConfigError {
    InvalidChildUser,
    InvalidShellPath,
    InvalidGuardianUnit,
    InvalidApprovedApp(String),
    DuplicateApprovedApp(String),
    Serialization(String),
}

fn valid_linux_user(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

fn valid_absolute_path(value: &str) -> bool {
    value.starts_with('/')
        && value.len() <= 4096
        && !value.contains('\0')
        && !value.contains('\n')
        && !value.contains('\r')
        && !value.contains('"')
}

fn valid_systemd_service_name(value: &str) -> bool {
    value.ends_with(".service")
        && value.len() <= 255
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '@' | '_' | '-' | '.' | ':'))
}

pub fn build_linux_platform_config(
    profile: &LinuxLockdownProfile,
) -> Result<PlatformLockdownConfig, LinuxLockdownConfigError> {
    profile.validate()?;
    let payload = serde_json::to_string(profile)
        .map_err(|error| LinuxLockdownConfigError::Serialization(error.to_string()))?;
    PlatformLockdownConfig::new(PlatformKind::Linux, payload)
        .map_err(|error| LinuxLockdownConfigError::Serialization(error.to_string()))
}

fn parse_linux_platform_config(
    config: &PlatformLockdownConfig,
) -> Result<LinuxLockdownProfile, PlatformAdapterError> {
    if config.platform() != PlatformKind::Linux {
        return Err(PlatformAdapterError::PlatformMismatch {
            expected: PlatformKind::Linux,
            received: config.platform(),
        });
    }

    let profile: LinuxLockdownProfile = serde_json::from_str(config.payload())
        .map_err(|error| PlatformAdapterError::InvalidConfiguration(error.to_string()))?;
    profile
        .validate()
        .map_err(|error| PlatformAdapterError::InvalidConfiguration(format!("{error:?}")))?;
    Ok(profile)
}

fn child_service_unit(profile: &LinuxLockdownProfile) -> String {
    format!(
        r#"[Unit]
Description=KidOS protected child session
Requires={guardian}
After={guardian} network-online.target
PartOf={target}

[Service]
Type=simple
User={user}
ExecStart="{shell}"
Restart=always
RestartSec=1
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=read-only
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
RestrictSUIDSGID=yes
LockPersonality=yes
RestrictRealtime=yes
CapabilityBoundingSet=
AmbientCapabilities=
UMask=0077

[Install]
WantedBy={target}
"#,
        guardian = profile.guardian_unit,
        target = KIDOS_CHILD_TARGET,
        user = profile.child_user,
        shell = profile.shell_path,
    )
}

fn child_target_unit(profile: &LinuxLockdownProfile) -> String {
    format!(
        r#"[Unit]
Description=KidOS protected child environment
Requires={guardian} {child}
After={guardian} {child}
AllowIsolate=yes
"#,
        guardian = profile.guardian_unit,
        child = KIDOS_CHILD_SERVICE,
    )
}

/// Closed set of privileged systemd operations that the Guardian host may perform.
///
/// The policy/service crate never executes operating-system commands. A privileged
/// host backend must map these variants to its audited platform mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemdAction {
    DaemonReload,
    EnableChildService,
    EnableChildTarget,
    StartChildTarget,
    StopChildTarget,
    DisableChildService,
    DisableChildTarget,
    IsChildServiceEnabled,
    IsChildTargetActive,
}

impl SystemdAction {
    pub const fn is_read_only(self) -> bool {
        matches!(
            self,
            Self::IsChildServiceEnabled | Self::IsChildTargetActive
        )
    }
}

pub trait LinuxPlatformBackend {
    fn ensure_privileged(&self) -> Result<(), PlatformAdapterError>;
    fn write_atomic(&mut self, path: &str, content: &str) -> Result<(), PlatformAdapterError>;
    fn remove_if_exists(&mut self, path: &str) -> Result<(), PlatformAdapterError>;
    fn file_exists(&self, path: &str) -> Result<bool, PlatformAdapterError>;
    fn systemd(&mut self, action: SystemdAction) -> Result<bool, PlatformAdapterError>;
    fn systemd_readonly(&self, action: SystemdAction) -> Result<bool, PlatformAdapterError>;
}

#[derive(Debug)]
pub struct LinuxPlatformAdapter<B: LinuxPlatformBackend> {
    backend: B,
}

impl<B: LinuxPlatformBackend> LinuxPlatformAdapter<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    fn rollback_partial_apply(&mut self) {
        let _ = self.backend.systemd(SystemdAction::StopChildTarget);
        let _ = self.backend.systemd(SystemdAction::DisableChildService);
        let _ = self.backend.systemd(SystemdAction::DisableChildTarget);
        let _ = self.backend.remove_if_exists(KIDOS_CHILD_SERVICE_PATH);
        let _ = self.backend.remove_if_exists(KIDOS_CHILD_TARGET_PATH);
        let _ = self.backend.remove_if_exists(KIDOS_LINUX_CONFIG_PATH);
        let _ = self.backend.systemd(SystemdAction::DaemonReload);
    }

    fn require_systemd(&mut self, action: SystemdAction) -> Result<(), PlatformAdapterError> {
        if action.is_read_only() {
            return Err(PlatformAdapterError::InvalidConfiguration(
                "read-only systemd action cannot be used as a mutating operation".into(),
            ));
        }

        if self.backend.systemd(action)? {
            Ok(())
        } else {
            Err(PlatformAdapterError::PlatformFailure(format!(
                "systemd action {action:?} failed"
            )))
        }
    }
}

impl<B: LinuxPlatformBackend> PlatformLockdownAdapter for LinuxPlatformAdapter<B> {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Linux
    }

    fn inspect(&self) -> Result<PlatformInspection, PlatformAdapterError> {
        if !self.backend.file_exists(KIDOS_LINUX_CONFIG_PATH)?
            || !self.backend.file_exists(KIDOS_CHILD_SERVICE_PATH)?
            || !self.backend.file_exists(KIDOS_CHILD_TARGET_PATH)?
        {
            return Ok(PlatformInspection::NotConfigured);
        }

        let enabled = self
            .backend
            .systemd_readonly(SystemdAction::IsChildServiceEnabled)?;
        let active = self
            .backend
            .systemd_readonly(SystemdAction::IsChildTargetActive)?;

        Ok(if enabled && active {
            PlatformInspection::Configured
        } else {
            PlatformInspection::NotConfigured
        })
    }

    fn apply(&mut self, config: &PlatformLockdownConfig) -> Result<(), PlatformAdapterError> {
        let profile = parse_linux_platform_config(config)?;
        self.backend.ensure_privileged()?;

        let result = (|| {
            self.backend
                .write_atomic(KIDOS_LINUX_CONFIG_PATH, config.payload())?;
            self.backend
                .write_atomic(KIDOS_CHILD_SERVICE_PATH, &child_service_unit(&profile))?;
            self.backend
                .write_atomic(KIDOS_CHILD_TARGET_PATH, &child_target_unit(&profile))?;

            self.require_systemd(SystemdAction::DaemonReload)?;
            self.require_systemd(SystemdAction::EnableChildService)?;
            self.require_systemd(SystemdAction::EnableChildTarget)?;
            self.require_systemd(SystemdAction::StartChildTarget)?;
            Ok(())
        })();

        if result.is_err() {
            self.rollback_partial_apply();
        }
        result
    }

    fn remove(&mut self) -> Result<(), PlatformAdapterError> {
        self.backend.ensure_privileged()?;

        let _ = self.backend.systemd(SystemdAction::StopChildTarget);
        let _ = self.backend.systemd(SystemdAction::DisableChildService);
        let _ = self.backend.systemd(SystemdAction::DisableChildTarget);

        self.backend.remove_if_exists(KIDOS_CHILD_SERVICE_PATH)?;
        self.backend.remove_if_exists(KIDOS_CHILD_TARGET_PATH)?;
        self.backend.remove_if_exists(KIDOS_LINUX_CONFIG_PATH)?;
        self.require_systemd(SystemdAction::DaemonReload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    #[derive(Debug, Default)]
    struct FakeLinuxBackend {
        privileged: bool,
        files: BTreeMap<String, String>,
        enabled: BTreeSet<String>,
        active: BTreeSet<String>,
        fail_start: bool,
    }

    impl FakeLinuxBackend {
        fn privileged() -> Self {
            Self {
                privileged: true,
                ..Self::default()
            }
        }

        fn execute(&mut self, action: SystemdAction) -> bool {
            match action {
                SystemdAction::DaemonReload => true,
                SystemdAction::EnableChildService => {
                    self.enabled.insert(KIDOS_CHILD_SERVICE.into());
                    true
                }
                SystemdAction::EnableChildTarget => {
                    self.enabled.insert(KIDOS_CHILD_TARGET.into());
                    true
                }
                SystemdAction::StartChildTarget => {
                    if self.fail_start {
                        false
                    } else {
                        self.active.insert(KIDOS_CHILD_TARGET.into());
                        true
                    }
                }
                SystemdAction::StopChildTarget => {
                    self.active.remove(KIDOS_CHILD_TARGET);
                    true
                }
                SystemdAction::DisableChildService => {
                    self.enabled.remove(KIDOS_CHILD_SERVICE);
                    true
                }
                SystemdAction::DisableChildTarget => {
                    self.enabled.remove(KIDOS_CHILD_TARGET);
                    true
                }
                SystemdAction::IsChildServiceEnabled => {
                    self.enabled.contains(KIDOS_CHILD_SERVICE)
                }
                SystemdAction::IsChildTargetActive => {
                    self.active.contains(KIDOS_CHILD_TARGET)
                }
            }
        }

        fn inspect(&self, action: SystemdAction) -> bool {
            match action {
                SystemdAction::IsChildServiceEnabled => {
                    self.enabled.contains(KIDOS_CHILD_SERVICE)
                }
                SystemdAction::IsChildTargetActive => {
                    self.active.contains(KIDOS_CHILD_TARGET)
                }
                _ => false,
            }
        }
    }

    impl LinuxPlatformBackend for FakeLinuxBackend {
        fn ensure_privileged(&self) -> Result<(), PlatformAdapterError> {
            if self.privileged {
                Ok(())
            } else {
                Err(PlatformAdapterError::AccessDenied)
            }
        }

        fn write_atomic(&mut self, path: &str, content: &str) -> Result<(), PlatformAdapterError> {
            self.files.insert(path.into(), content.into());
            Ok(())
        }

        fn remove_if_exists(&mut self, path: &str) -> Result<(), PlatformAdapterError> {
            self.files.remove(path);
            Ok(())
        }

        fn file_exists(&self, path: &str) -> Result<bool, PlatformAdapterError> {
            Ok(self.files.contains_key(path))
        }

        fn systemd(&mut self, action: SystemdAction) -> Result<bool, PlatformAdapterError> {
            if action.is_read_only() {
                return Err(PlatformAdapterError::InvalidConfiguration(
                    "read-only action used through mutable backend method".into(),
                ));
            }
            Ok(self.execute(action))
        }

        fn systemd_readonly(&self, action: SystemdAction) -> Result<bool, PlatformAdapterError> {
            if !action.is_read_only() {
                return Err(PlatformAdapterError::InvalidConfiguration(
                    "mutating action used through read-only backend method".into(),
                ));
            }
            Ok(self.inspect(action))
        }
    }

    fn profile() -> LinuxLockdownProfile {
        LinuxLockdownProfile {
            child_user: "kidos-child".into(),
            shell_path: "/opt/kidos/bin/kidos-shell".into(),
            guardian_unit: "kidos-guardian.service".into(),
            approved_apps: vec!["/opt/kidos/apps/zigazoo".into()],
        }
    }

    #[test]
    fn builds_linux_platform_configuration() {
        let config = build_linux_platform_config(&profile()).unwrap();
        assert_eq!(config.platform(), PlatformKind::Linux);
        assert!(config.payload().contains("kidos-child"));
    }

    #[test]
    fn rejects_admin_or_malformed_profile_values() {
        let mut invalid = profile();
        invalid.child_user = "root/user".into();
        assert!(invalid.validate().is_err());

        invalid = profile();
        invalid.shell_path = "relative/kidos-shell".into();
        assert!(invalid.validate().is_err());

        invalid = profile();
        invalid.guardian_unit = "kidos guardian.service".into();
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn applies_systemd_child_containment_files() {
        let backend = FakeLinuxBackend::privileged();
        let mut adapter = LinuxPlatformAdapter::new(backend);
        let config = build_linux_platform_config(&profile()).unwrap();

        adapter.apply(&config).unwrap();

        assert!(adapter.backend.files.contains_key(KIDOS_LINUX_CONFIG_PATH));
        assert!(adapter.backend.files.contains_key(KIDOS_CHILD_SERVICE_PATH));
        assert!(adapter.backend.files.contains_key(KIDOS_CHILD_TARGET_PATH));
        assert!(adapter.backend.enabled.contains(KIDOS_CHILD_SERVICE));
        assert!(adapter.backend.active.contains(KIDOS_CHILD_TARGET));
    }

    #[test]
    fn generic_platform_service_can_drive_linux_adapter() {
        let backend = FakeLinuxBackend::privileged();
        let adapter = LinuxPlatformAdapter::new(backend);
        let mut service = crate::platform::PlatformLockdownService::new(adapter);
        let config = build_linux_platform_config(&profile()).unwrap();

        service.prepare_and_apply(&config).unwrap();
        assert_eq!(
            service.status(0).state,
            crate::platform::PlatformLockdownState::Locked
        );
    }

    #[test]
    fn failed_start_rolls_back_partial_linux_configuration() {
        let mut backend = FakeLinuxBackend::privileged();
        backend.fail_start = true;
        let mut adapter = LinuxPlatformAdapter::new(backend);
        let config = build_linux_platform_config(&profile()).unwrap();

        assert!(adapter.apply(&config).is_err());
        assert!(!adapter.backend.files.contains_key(KIDOS_LINUX_CONFIG_PATH));
        assert!(!adapter.backend.files.contains_key(KIDOS_CHILD_SERVICE_PATH));
        assert!(!adapter.backend.files.contains_key(KIDOS_CHILD_TARGET_PATH));
        assert!(!adapter.backend.enabled.contains(KIDOS_CHILD_SERVICE));
        assert!(!adapter.backend.enabled.contains(KIDOS_CHILD_TARGET));
    }

    #[test]
    fn unprivileged_apply_is_denied() {
        let backend = FakeLinuxBackend::default();
        let mut adapter = LinuxPlatformAdapter::new(backend);
        let config = build_linux_platform_config(&profile()).unwrap();

        assert_eq!(adapter.apply(&config), Err(PlatformAdapterError::AccessDenied));
    }

    #[test]
    fn read_only_systemd_actions_are_separated_from_mutations() {
        assert!(SystemdAction::IsChildServiceEnabled.is_read_only());
        assert!(SystemdAction::IsChildTargetActive.is_read_only());
        assert!(!SystemdAction::StartChildTarget.is_read_only());
        assert!(!SystemdAction::DaemonReload.is_read_only());
    }
}
