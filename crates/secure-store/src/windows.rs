use keyring::{Entry, Error as KeyringError};

use crate::{hash_secret, verify_hash, SecretStore, SecureStoreError};

pub struct WindowsSecretStore {
    service: String,
}

impl WindowsSecretStore {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    fn entry(&self, key: &str) -> Result<Entry, SecureStoreError> {
        Entry::new(&self.service, key).map_err(|error| SecureStoreError::Backend(error.to_string()))
    }
}

impl SecretStore for WindowsSecretStore {
    fn put_secret(&self, key: &str, secret: &str) -> Result<(), SecureStoreError> {
        let verifier = hash_secret(secret)?;
        self.entry(key)?
            .set_password(&verifier)
            .map_err(|error| SecureStoreError::Backend(error.to_string()))
    }

    fn verify_secret(&self, key: &str, candidate: &str) -> Result<bool, SecureStoreError> {
        let entry = self.entry(key)?;
        match entry.get_password() {
            Ok(verifier) => verify_hash(&verifier, candidate),
            Err(KeyringError::NoEntry) => Ok(false),
            Err(error) => Err(SecureStoreError::Backend(error.to_string())),
        }
    }

    fn delete_secret(&self, key: &str) -> Result<(), SecureStoreError> {
        let entry = self.entry(key)?;
        match entry.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(error) => Err(SecureStoreError::Backend(error.to_string())),
        }
    }
}
