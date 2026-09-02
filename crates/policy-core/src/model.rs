#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Block,
    RequireParent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteCategory {
    Unknown,
    Approved,
    Educational,
    Prohibited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Normal,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadMode {
    BlockAll,
    RequireParent,
    AllowSafe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationContext {
    pub domain: String,
    pub age: u8,
    pub parent_blocked: bool,
    pub parent_allowed: bool,
    pub category: SiteCategory,
    pub risk: RiskLevel,
    pub unknown_web_enabled: bool,
}

impl NavigationContext {
    pub fn new(domain: impl Into<String>, age: u8) -> Self {
        Self {
            domain: domain.into(),
            age,
            parent_blocked: false,
            parent_allowed: false,
            category: SiteCategory::Unknown,
            risk: RiskLevel::Normal,
            unknown_web_enabled: false,
        }
    }

    pub fn with_parent_blocked(mut self, value: bool) -> Self { self.parent_blocked = value; self }
    pub fn with_parent_allowed(mut self, value: bool) -> Self { self.parent_allowed = value; self }
    pub fn with_category(mut self, value: SiteCategory) -> Self { self.category = value; self }
    pub fn with_risk(mut self, value: RiskLevel) -> Self { self.risk = value; self }
    pub fn with_unknown_web_enabled(mut self, value: bool) -> Self { self.unknown_web_enabled = value; self }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadContext {
    pub file_name: String,
    pub mime_type: String,
    pub age: u8,
    pub parent_blocked: bool,
    pub parent_allowed: bool,
    pub download_mode: DownloadMode,
}

impl DownloadContext {
    pub fn new(file_name: impl Into<String>, mime_type: impl Into<String>, age: u8) -> Self {
        Self {
            file_name: file_name.into(),
            mime_type: mime_type.into(),
            age,
            parent_blocked: false,
            parent_allowed: false,
            download_mode: DownloadMode::RequireParent,
        }
    }

    pub fn with_parent_blocked(mut self, value: bool) -> Self { self.parent_blocked = value; self }
    pub fn with_parent_allowed(mut self, value: bool) -> Self { self.parent_allowed = value; self }
    pub fn with_download_mode(mut self, value: DownloadMode) -> Self { self.download_mode = value; self }
}
