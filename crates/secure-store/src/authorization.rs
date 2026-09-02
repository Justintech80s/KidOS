use crate::{PinAttemptLimiter, SecretStore, SecureStoreError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentAuthorizationResult {
    Authorized,
    Denied,
    Locked,
}

pub struct ParentAuthorization<S> {
    store: S,
    key: String,
    limiter: PinAttemptLimiter,
}

impl<S: SecretStore> ParentAuthorization<S> {
    pub fn new(store: S, key: impl Into<String>) -> Self {
        Self {
            store,
            key: key.into(),
            limiter: PinAttemptLimiter::default(),
        }
    }

    pub fn verify(
        &mut self,
        candidate: &str,
        now_seconds: u64,
    ) -> Result<ParentAuthorizationResult, SecureStoreError> {
        if self.limiter.is_locked(now_seconds) {
            return Ok(ParentAuthorizationResult::Locked);
        }

        if self.store.verify_secret(&self.key, candidate)? {
            self.limiter.record_success();
            return Ok(ParentAuthorizationResult::Authorized);
        }

        self.limiter.record_failure(now_seconds)?;
        if self.limiter.is_locked(now_seconds) {
            Ok(ParentAuthorizationResult::Locked)
        } else {
            Ok(ParentAuthorizationResult::Denied)
        }
    }
}
