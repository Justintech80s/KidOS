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

#[cfg(target_os = "windows")]
#[derive(Debug, Default)]
pub struct WindowsAssignedAccessAdapter;

#[cfg(target_os = "windows")]
impl WindowsLockdownAdapter for WindowsAssignedAccessAdapter {
    fn inspect(&self) -> Result<LockdownInspection, LockdownAdapterError> {
        // The privileged Guardian host will bind this narrow adapter to the
        // Windows AssignedAccess CSP/MDM Bridge. No renderer command execution
        // is exposed through this type.
        Err(LockdownAdapterError::PlatformFailure("Assigned Access host binding not initialized".into()))
    }

    fn apply(&mut self, _config: &AssignedAccessConfig) -> Result<(), LockdownAdapterError> {
        Err(LockdownAdapterError::PlatformFailure("Assigned Access host binding not initialized".into()))
    }

    fn remove(&mut self) -> Result<(), LockdownAdapterError> {
        Err(LockdownAdapterError::PlatformFailure("Assigned Access host binding not initialized".into()))
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
