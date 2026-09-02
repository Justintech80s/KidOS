use std::{collections::HashMap, sync::Mutex};

use crate::{hash_secret, verify_hash, SecretStore, SecureStoreError};

#[derive(Default)]
pub struct MemorySecretStore {
    secrets: Mutex<HashMap<String, String>>,
}

impl SecretStore for MemorySecretStore {
    fn put_secret(&self, key: &str, secret: &str) -> Result<(), SecureStoreError> {
        let verifier = hash_secret(secret)?;
        self.secrets
            .lock()
            .map_err(|_| SecureStoreError::Backend("secure store lock poisoned".into()))?
            .insert(key.to_owned(), verifier);
        Ok(())
    }

    fn verify_secret(&self, key: &str, candidate: &str) -> Result<bool, SecureStoreError> {
        let secrets = self
            .secrets
            .lock()
            .map_err(|_| SecureStoreError::Backend("secure store lock poisoned".into()))?;
        let Some(verifier) = secrets.get(key) else {
            return Ok(false);
        };
        verify_hash(verifier, candidate)
    }

    fn delete_secret(&self, key: &str) -> Result<(), SecureStoreError> {
        self.secrets
            .lock()
            .map_err(|_| SecureStoreError::Backend("secure store lock poisoned".into()))?
            .remove(key);
        Ok(())
    }
}
