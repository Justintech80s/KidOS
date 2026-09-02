mod memory;
#[cfg(target_os = "windows")]
mod windows;

use std::{error::Error, fmt};

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;

pub use memory::MemorySecretStore;
#[cfg(target_os = "windows")]
pub use windows::WindowsSecretStore;

pub trait SecretStore: Send + Sync {
    fn put_secret(&self, key: &str, secret: &str) -> Result<(), SecureStoreError>;
    fn verify_secret(&self, key: &str, candidate: &str) -> Result<bool, SecureStoreError>;
    fn delete_secret(&self, key: &str) -> Result<(), SecureStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecureStoreError {
    Hash(String),
    Backend(String),
}

impl fmt::Display for SecureStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hash(message) => write!(formatter, "secret verifier error: {message}"),
            Self::Backend(message) => write!(formatter, "secure store backend error: {message}"),
        }
    }
}

impl Error for SecureStoreError {}

pub(crate) fn hash_secret(secret: &str) -> Result<String, SecureStoreError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| SecureStoreError::Hash(error.to_string()))
}

pub(crate) fn verify_hash(verifier: &str, candidate: &str) -> Result<bool, SecureStoreError> {
    let parsed = PasswordHash::new(verifier)
        .map_err(|error| SecureStoreError::Hash(error.to_string()))?;
    Ok(Argon2::default()
        .verify_password(candidate.as_bytes(), &parsed)
        .is_ok())
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PinAttemptLimiter {
    failed_attempts: u8,
    locked_until: Option<u64>,
}

impl PinAttemptLimiter {
    pub const MAX_FAILED_ATTEMPTS: u8 = 5;
    pub const LOCKOUT_SECONDS: u64 = 60;

    pub fn record_failure(&mut self, now_seconds: u64) -> Result<(), SecureStoreError> {
        if self.is_locked(now_seconds) {
            return Ok(());
        }

        self.failed_attempts = self.failed_attempts.saturating_add(1);
        if self.failed_attempts >= Self::MAX_FAILED_ATTEMPTS {
            self.failed_attempts = 0;
            self.locked_until = Some(now_seconds.saturating_add(Self::LOCKOUT_SECONDS));
        }
        Ok(())
    }

    pub fn record_success(&mut self) {
        self.failed_attempts = 0;
        self.locked_until = None;
    }

    pub fn is_locked(&self, now_seconds: u64) -> bool {
        self.locked_until
            .map(|locked_until| now_seconds < locked_until)
            .unwrap_or(false)
    }
}
