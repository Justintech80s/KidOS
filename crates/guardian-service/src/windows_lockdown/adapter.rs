#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignedAccessConfig {
    xml: String,
}

impl AssignedAccessConfig {
    pub fn validated(xml: String) -> Self {
        Self { xml }
    }

    pub fn as_xml(&self) -> &str {
        &self.xml
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockdownInspection {
    NotConfigured,
    Configured,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockdownAdapterError {
    AccessDenied,
    UnsupportedPlatform,
    PlatformFailure(String),
}

pub trait WindowsLockdownAdapter {
    fn inspect(&self) -> Result<LockdownInspection, LockdownAdapterError>;
    fn apply(&mut self, config: &AssignedAccessConfig) -> Result<(), LockdownAdapterError>;
    fn remove(&mut self) -> Result<(), LockdownAdapterError>;
}

#[derive(Debug, Default)]
pub struct InMemoryWindowsLockdownAdapter {
    configured: bool,
    failure: Option<LockdownAdapterError>,
}

impl InMemoryWindowsLockdownAdapter {
    pub fn with_failure(error: LockdownAdapterError) -> Self {
        Self { configured: false, failure: Some(error) }
    }

    fn fail_if_configured(&self) -> Result<(), LockdownAdapterError> {
        match &self.failure {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }
}

impl WindowsLockdownAdapter for InMemoryWindowsLockdownAdapter {
    fn inspect(&self) -> Result<LockdownInspection, LockdownAdapterError> {
        self.fail_if_configured()?;
        Ok(if self.configured { LockdownInspection::Configured } else { LockdownInspection::NotConfigured })
    }

    fn apply(&mut self, _config: &AssignedAccessConfig) -> Result<(), LockdownAdapterError> {
        self.fail_if_configured()?;
        self.configured = true;
        Ok(())
    }

    fn remove(&mut self) -> Result<(), LockdownAdapterError> {
        self.fail_if_configured()?;
        self.configured = false;
        Ok(())
    }
}

fn encode_for_mdm_bridge(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(target_os = "windows")]
mod production {
    use super::{
        encode_for_mdm_bridge, AssignedAccessConfig, LockdownAdapterError, LockdownInspection,
    };
    use std::{mem::size_of, ptr::null_mut};
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        Security::{
            GetTokenInformation, IsWellKnownSid, TOKEN_QUERY, TOKEN_USER, TokenUser,
            WinLocalSystemSid,
        },
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    use wmi::{IWbemClassWrapper, Variant, WMIConnection};

    const MDM_NAMESPACE: &str = r"root\cimv2\mdm\dmmap";
    const ASSIGNED_ACCESS_QUERY: &str = "SELECT * FROM MDM_AssignedAccess";

    fn platform_failure(error: impl ToString) -> LockdownAdapterError {
        LockdownAdapterError::PlatformFailure(error.to_string())
    }

    pub fn ensure_local_system() -> Result<(), LockdownAdapterError> {
        unsafe {
            let mut token = null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
                return Err(LockdownAdapterError::AccessDenied);
            }

            let mut needed = 0u32;
            let _ = GetTokenInformation(token, TokenUser, null_mut(), 0, &mut needed);
            if needed == 0 {
                CloseHandle(token);
                return Err(LockdownAdapterError::AccessDenied);
            }

            let word = size_of::<usize>();
            let words = (needed as usize + word - 1) / word;
            let mut buffer = vec![0usize; words];
            if GetTokenInformation(
                token,
                TokenUser,
                buffer.as_mut_ptr().cast(),
                needed,
                &mut needed,
            ) == 0
            {
                CloseHandle(token);
                return Err(LockdownAdapterError::AccessDenied);
            }

            let token_user = &*(buffer.as_ptr().cast::<TOKEN_USER>());
            let is_system = IsWellKnownSid(token_user.User.Sid, WinLocalSystemSid) != 0;
            CloseHandle(token);

            if is_system {
                Ok(())
            } else {
                Err(LockdownAdapterError::AccessDenied)
            }
        }
    }

    fn connection() -> Result<WMIConnection, LockdownAdapterError> {
        WMIConnection::with_namespace_path(MDM_NAMESPACE).map_err(platform_failure)
    }

    fn assigned_access_instance(
        connection: &WMIConnection,
    ) -> Result<Option<IWbemClassWrapper>, LockdownAdapterError> {
        let mut objects = connection
            .exec_query(ASSIGNED_ACCESS_QUERY)
            .map_err(platform_failure)?;
        match objects.next() {
            Some(Ok(object)) => Ok(Some(object)),
            Some(Err(error)) => Err(platform_failure(error)),
            None => Ok(None),
        }
    }

    pub fn inspect() -> Result<LockdownInspection, LockdownAdapterError> {
        ensure_local_system()?;
        let connection = connection()?;
        let Some(instance) = assigned_access_instance(&connection)? else {
            return Ok(LockdownInspection::NotConfigured);
        };

        match instance.get_property("Configuration").map_err(platform_failure)? {
            Variant::String(configuration) if !configuration.trim().is_empty() => {
                Ok(LockdownInspection::Configured)
            }
            _ => Ok(LockdownInspection::NotConfigured),
        }
    }

    pub fn apply(config: &AssignedAccessConfig) -> Result<(), LockdownAdapterError> {
        ensure_local_system()?;
        let connection = connection()?;
        let instance = assigned_access_instance(&connection)?.ok_or_else(|| {
            LockdownAdapterError::PlatformFailure(
                "MDM_AssignedAccess provider instance is unavailable".into(),
            )
        })?;

        instance
            .put_property("Configuration", encode_for_mdm_bridge(config.as_xml()))
            .map_err(platform_failure)?;
        connection.put_instance(&instance).map_err(platform_failure)
    }

    pub fn remove() -> Result<(), LockdownAdapterError> {
        ensure_local_system()?;
        let connection = connection()?;
        let Some(instance) = assigned_access_instance(&connection)? else {
            return Ok(());
        };
        let path = instance.path().map_err(platform_failure)?;
        connection.delete_instance(&path).map_err(platform_failure)
    }
}

#[cfg(target_os = "windows")]
#[derive(Debug, Default)]
pub struct WindowsAssignedAccessAdapter;

#[cfg(target_os = "windows")]
impl WindowsLockdownAdapter for WindowsAssignedAccessAdapter {
    fn inspect(&self) -> Result<LockdownInspection, LockdownAdapterError> {
        production::inspect()
    }

    fn apply(&mut self, config: &AssignedAccessConfig) -> Result<(), LockdownAdapterError> {
        production::apply(config)
    }

    fn remove(&mut self) -> Result<(), LockdownAdapterError> {
        production::remove()
    }
}

#[cfg(not(target_os = "windows"))]
#[derive(Debug, Default)]
pub struct WindowsAssignedAccessAdapter;

#[cfg(not(target_os = "windows"))]
impl WindowsLockdownAdapter for WindowsAssignedAccessAdapter {
    fn inspect(&self) -> Result<LockdownInspection, LockdownAdapterError> {
        Ok(LockdownInspection::Unsupported)
    }

    fn apply(&mut self, _config: &AssignedAccessConfig) -> Result<(), LockdownAdapterError> {
        Err(LockdownAdapterError::UnsupportedPlatform)
    }

    fn remove(&mut self) -> Result<(), LockdownAdapterError> {
        Err(LockdownAdapterError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use super::encode_for_mdm_bridge;

    #[test]
    fn assigned_access_xml_is_encoded_once_for_mdm_bridge_transport() {
        assert_eq!(
            encode_for_mdm_bridge(r#"<Config A="x&y">'z'</Config>"#),
            "&lt;Config A=&quot;x&amp;y&quot;&gt;&#39;z&#39;&lt;/Config&gt;"
        );
    }
}
