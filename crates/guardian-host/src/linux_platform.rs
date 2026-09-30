use guardian_service::{
    linux_lockdown::{
        LinuxPlatformAdapter, LinuxPlatformBackend, SystemdAction, KIDOS_CHILD_SERVICE,
        KIDOS_CHILD_SERVICE_PATH, KIDOS_CHILD_TARGET, KIDOS_CHILD_TARGET_PATH,
        KIDOS_LINUX_CONFIG_PATH,
    },
    PlatformAdapterError,
};
use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
};

#[derive(Debug, Default)]
pub struct GuardianLinuxBackend;

impl GuardianLinuxBackend {
    fn systemd_args(action: SystemdAction) -> &'static [&'static str] {
        match action {
            SystemdAction::DaemonReload => &["daemon-reload"],
            SystemdAction::EnableChildService => &["enable", KIDOS_CHILD_SERVICE],
            SystemdAction::EnableChildTarget => &["enable", KIDOS_CHILD_TARGET],
            SystemdAction::StartChildTarget => &["start", KIDOS_CHILD_TARGET],
            SystemdAction::StopChildTarget => &["stop", KIDOS_CHILD_TARGET],
            SystemdAction::DisableChildService => &["disable", KIDOS_CHILD_SERVICE],
            SystemdAction::DisableChildTarget => &["disable", KIDOS_CHILD_TARGET],
            SystemdAction::IsChildServiceEnabled => {
                &["is-enabled", "--quiet", KIDOS_CHILD_SERVICE]
            }
            SystemdAction::IsChildTargetActive => {
                &["is-active", "--quiet", KIDOS_CHILD_TARGET]
            }
        }
    }

    fn run_systemd(action: SystemdAction) -> Result<bool, PlatformAdapterError> {
        let status = Command::new("systemctl")
            .args(Self::systemd_args(action))
            .status()
            .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))?;
        Ok(status.success())
    }

    fn file_mode(path: &str) -> u32 {
        if path == KIDOS_LINUX_CONFIG_PATH {
            0o600
        } else {
            0o644
        }
    }
}

impl LinuxPlatformBackend for GuardianLinuxBackend {
    fn ensure_privileged(&self) -> Result<(), PlatformAdapterError> {
        let status = fs::read_to_string("/proc/self/status")
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
        let destination = Path::new(path);
        let parent = destination.parent().ok_or_else(|| {
            PlatformAdapterError::InvalidConfiguration(
                "Linux destination has no parent directory".into(),
            )
        })?;
        fs::create_dir_all(parent)
            .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))?;

        let temporary = parent.join(format!(".kidos-{}.tmp", std::process::id()));
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)
            .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))?;
        file.write_all(content.as_bytes())
            .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))?;
        file.sync_all()
            .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))?;
        fs::set_permissions(
            &temporary,
            fs::Permissions::from_mode(Self::file_mode(path)),
        )
        .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))?;
        fs::rename(&temporary, destination)
            .map_err(|error| PlatformAdapterError::PlatformFailure(error.to_string()))
    }

    fn remove_if_exists(&mut self, path: &str) -> Result<(), PlatformAdapterError> {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(PlatformAdapterError::PlatformFailure(error.to_string())),
        }
    }

    fn file_exists(&self, path: &str) -> Result<bool, PlatformAdapterError> {
        match fs::symlink_metadata(path) {
            Ok(metadata) => Ok(metadata.file_type().is_file()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(PlatformAdapterError::PlatformFailure(error.to_string())),
        }
    }

    fn systemd(&mut self, action: SystemdAction) -> Result<bool, PlatformAdapterError> {
        if action.is_read_only() {
            return Err(PlatformAdapterError::InvalidConfiguration(
                "read-only systemd action cannot use the mutating Guardian path".into(),
            ));
        }
        Self::run_systemd(action)
    }

    fn systemd_readonly(&self, action: SystemdAction) -> Result<bool, PlatformAdapterError> {
        if !action.is_read_only() {
            return Err(PlatformAdapterError::InvalidConfiguration(
                "mutating systemd action cannot use the read-only Guardian path".into(),
            ));
        }
        Self::run_systemd(action)
    }
}

pub type ProductionLinuxPlatformAdapter = LinuxPlatformAdapter<GuardianLinuxBackend>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systemd_commands_are_a_closed_allowlist() {
        assert_eq!(
            GuardianLinuxBackend::systemd_args(SystemdAction::DaemonReload),
            &["daemon-reload"]
        );
        assert_eq!(
            GuardianLinuxBackend::systemd_args(SystemdAction::EnableChildService),
            &["enable", KIDOS_CHILD_SERVICE]
        );
        assert_eq!(
            GuardianLinuxBackend::systemd_args(SystemdAction::EnableChildTarget),
            &["enable", KIDOS_CHILD_TARGET]
        );
        assert_eq!(
            GuardianLinuxBackend::systemd_args(SystemdAction::StartChildTarget),
            &["start", KIDOS_CHILD_TARGET]
        );
        assert_eq!(
            GuardianLinuxBackend::systemd_args(SystemdAction::StopChildTarget),
            &["stop", KIDOS_CHILD_TARGET]
        );
        assert_eq!(
            GuardianLinuxBackend::systemd_args(SystemdAction::DisableChildService),
            &["disable", KIDOS_CHILD_SERVICE]
        );
        assert_eq!(
            GuardianLinuxBackend::systemd_args(SystemdAction::DisableChildTarget),
            &["disable", KIDOS_CHILD_TARGET]
        );
        assert_eq!(
            GuardianLinuxBackend::systemd_args(SystemdAction::IsChildServiceEnabled),
            &["is-enabled", "--quiet", KIDOS_CHILD_SERVICE]
        );
        assert_eq!(
            GuardianLinuxBackend::systemd_args(SystemdAction::IsChildTargetActive),
            &["is-active", "--quiet", KIDOS_CHILD_TARGET]
        );
    }

    #[test]
    fn protected_policy_file_is_not_world_readable() {
        assert_eq!(GuardianLinuxBackend::file_mode(KIDOS_LINUX_CONFIG_PATH), 0o600);
        assert_eq!(GuardianLinuxBackend::file_mode(KIDOS_CHILD_SERVICE_PATH), 0o644);
        assert_eq!(GuardianLinuxBackend::file_mode(KIDOS_CHILD_TARGET_PATH), 0o644);
    }
}
