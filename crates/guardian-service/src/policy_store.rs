#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicySnapshot {
    pub id: String,
    pub integrity_valid: bool,
    pub source: String,
}

impl PolicySnapshot {
    pub fn valid(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            integrity_valid: true,
            source: String::new(),
        }
    }

    pub fn invalid(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            integrity_valid: false,
            source: String::new(),
        }
    }

    pub(crate) fn with_source(mut self, source: &str) -> Self {
        self.source = source.to_string();
        self
    }

    pub(crate) fn strict_baseline() -> Self {
        Self {
            id: "strict-baseline".into(),
            integrity_valid: true,
            source: "strict_baseline".into(),
        }
    }
}
