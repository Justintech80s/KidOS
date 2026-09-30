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

pub trait LinuxPlatformBackend {
    fn ensure_privileged(&self) -> Result<(), PlatformAdapterError>;
    fn write_atomic(&mut self, path: &str, content: &str) -> Result<(), PlatformAdapterError>;
    fn remove_if_exists(&mut self, path: &str) -> Result<(), PlatformAdapterError>;
    fn file_exists(&self, path: &str) -> Result<bool, PlatformAdapterError>;
    fn systemctl(&mut self, args: &[&str]) -> Result<bool, PlatformAdapterError>;
    fn systemctl_readonly(&self, args: &[&str]) -> Result<bool, PlatformAdapterError>;
}

#[derive(Debug, Default)]
pub struct SystemLinuxBackend;

#[cfg(target_os = "linux")]
impl LinuxPlatformBackend for SystemLinuxBackend {
    fn ensure_privileged(&self) -> Result<(), PlatformAdapterError> {
        let status = std::fs::read_to_string("/proc/self/status")
            .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))?;
        let effective_uid = status
            .lines()
            .find(|line| line.starts_with("Uid:"))
            .and_then(|line| line.split_whitespace().nth(2))
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| {
                PlatformAdapterError::PlatformFailure(
                    "KidOS could not determine the effective Linux user id".into(),
                )
            })?;

        if effective_uid == 0 {
            Ok(())
        } else {
            Err(PlatformAdapterError::AccessDenied)
        }
    }

    fn write_atomic(&mut self, path: &str, content: &str) -> Result<(), PlatformAdapterError> {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;

        let destination = std::path::Path::new(path);
        let parent = destination.parent().ok_or_else(|| {
            PlatformAdapterError::InvalidConfiguration("Linux destination has no parent".into())
        })?;
        std::fs::create_dir_all(parent)
            .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))?;

        let temporary = parent.join(format!(
            ".kidos-{}.tmp",
            std::process::id()
        ));
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)
            .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))?;
        file.write_all(content.as_bytes())
            .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))?;
        file.sync_all()
            .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))?;
        std::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o644))
            .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))?;
        std::fs::rename(&temporary, destination)
            .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))
    }

    fn remove_if_exists(&mut self, path: &str) -> Result<(), PlatformAdapterError> {
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(PlatformAdapterError::PlatformFailure(error.to_string())),
        }
    }

    fn file_exists(&self, path: &str) -> Result<bool, PlatformAdapterError> {
        match std::fs::metadata(path) {
            Ok(metadata) => Ok(metadata.is_file()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(PlatformAdapterError::PlatformFailure(error.to_string())),
        }
    }

    fn systemctl(&mut self, args: &[&str]) -> Result<bool, PlatformAdapterError> {
        self.systemctl_readonly(args)
    }

    fn systemctl_readonly(&self, args: &[&str]) -> Result<bool, PlatformAdapterError> {
        let status = std::process::Command::new("systemctl")
            .args(args)
            .status()
            .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))?;
        Ok(status.success())
    }
}

#[cfg(not(target_os = "linux"))]
impl LinuxPlatformBackend for SystemLinuxBackend {
    fn ensure_privileged(&self) -> Result<(), PlatformAdapterError> {
        Err(PlatformAdapterError::UnsupportedPlatform)
    }

    fn write_atomic(&mut self, _path: &str, _content: &str) -> Result<(), PlatformAdapterError> {
        Err(PlatformAdapterError::UnsupportedPlatform)
    }

    fn remove_if_exists(&mut self, _path: &str) -> Result<(), PlatformAdapterError> {
        Err(PlatformAdapterError::UnsupportedPlatform)
    }

    fn file_exists(&self, _path: &str) -> Result<bool, PlatformAdapterError> {
        Err(PlatformAdapterError::UnsupportedPlatform)
    }

    fn systemctl(&mut self, _args: &[&str]) -> Result<bool, PlatformAdapterError> {
        Err(PlatformAdapterError::UnsupportedPlatform)
    }

    fn systemctl_readonly(&self, _args: &[&str]) -> Result<bool, PlatformAdapterError> {
        Err(PlatformAdapterError::UnsupportedPlatform)
    }
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
        let _ = self.backend.systemctl(&["stop", KIDOS_CHILD_TARGET]);
        let _ = self.backend.systemctl(&["disable", KIDOS_CHILD_SERVICE]);
        let _ = self.backend.systemctl(&["disable", KIDOS_CHILD_TARGET]);
        let _ = self.backend.remove_if_exists(KIDOS_CHILD_SERVICE_PATH);
        let _ = self.backend.remove_if_exists(KIDOS_CHILD_TARGET_PATH);
        let _ = self.backend.remove_if_exists(KIDOS_LINUX_CONFIG_PATH);
        let _ = self.backend.systemctl(&["daemon-reload"]);
    }

    fn require_systemctl(&mut self, args: &[&str]) -> Result<(), PlatformAdapterError> {
        if self.backend.systemctl(args)? {
            Ok(())
        } else {
            Err(PlatformAdapterError::PlatformFailure(format!(
                "systemctl {:?} failed",
                args
            )))
        }
    }
}

impl<B> Default for LinuxPlatformAdapter<B>
where
    B: LinuxPlatformBackend + Default,
{
    fn default() -> Self {
        Self::new(B::default())
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
            .systemctl_readonly(&["is-enabled", "--quiet", KIDOS_CHILD_SERVICE])?;
        let active = self
            .backend
            .systemctl_readonly(&["is-active", "--quiet", KIDOS_CHILD_TARGET])?;

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

            self.require_systemctl(&["daemon-reload"])?;
            self.require_systemctl(&["enable", KIDOS_CHILD_SERVICE])?;
            self.require_systemctl(&["enable", KIDOS_CHILD_TARGET])?;
            self.require_systemctl(&["start", KIDOS_CHILD_TARGET])?;
            Ok(())
        })();

        if result.is_err() {
            self.rollback_partial_apply();
        }
        result
    }

    fn remove(&mut self) -> Result<(), PlatformAdapterError> {
        self.backend.ensure_privileged()?;

        let _ = self.backend.systemctl(&["stop", KIDOS_CHILD_TARGET]);
        let _ = self.backend.systemctl(&["disable", KIDOS_CHILD_SERVICE]);
        let _ = self.backend.systemctl(&["disable", KIDOS_CHILD_TARGET]);

        self.backend.remove_if_exists(KIDOS_CHILD_SERVICE_PATH)?;
        self.backend.remove_if_exists(KIDOS_CHILD_TARGET_PATH)?;
        self.backend.remove_if_exists(KIDOS_LINUX_CONFIG_PATH)?;
        self.require_systemctl(&["daemon-reload"])
    }
}

pub type ProductionLinuxPlatformAdapter = LinuxPlatformAdapter<SystemLinuxBackend>;

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

        fn systemctl(&mut self, args: &[&str]) -> Result<bool, PlatformAdapterError> {
            match args {
                ["daemon-reload"] => Ok(true),
                ["enable", unit] => {
                    self.enabled.insert((*unit).into());
                    Ok(true)
                }
                ["disable", unit] => {
                    self.enabled.remove(*unit);
                    Ok(true)
                }
                ["start", unit] if *unit == KIDOS_CHILD_TARGET => {
                    if self.fail_start {
                        Ok(false)
                    } else {
                        self.active.insert((*unit).into());
                        Ok(true)
                    }
                }
                ["stop", unit] => {
                    self.active.remove(*unit);
                    Ok(true)
                }
                ["is-enabled", "--quiet", unit] => Ok(self.enabled.contains(*unit)),
                ["is-active", "--quiet", unit] => Ok(self.active.contains(*unit)),
                _ => Ok(false),
            }
        }

        fn systemctl_readonly(&self, args: &[&str]) -> Result<bool, PlatformAdapterError> {
            match args {
                ["is-enabled", "--quiet", unit] => Ok(self.enabled.contains(*unit)),
                ["is-active", "--quiet", unit] => Ok(self.active.contains(*unit)),
                _ => Ok(false),
            }
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
    }

    #[test]
    fn unprivileged_apply_is_denied() {
        let backend = FakeLinuxBackend::default();
        let mut adapter = LinuxPlatformAdapter::new(backend);
        let config = build_linux_platform_config(&profile()).unwrap();

        assert_eq!(adapter.apply(&config), Err(PlatformAdapterError::AccessDenied));
    }
}
