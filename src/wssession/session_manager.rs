use crate::{Error, Result};
use std::sync::atomic::{AtomicBool, Ordering};

/// A singleton manager to track session existence.
#[derive(Default, Debug)]
pub struct SessionManager {
    session_exists: AtomicBool,
}

impl SessionManager {

    /// Attempts to acquire a session.
    /// Returns `Ok(())` if successful; otherwise, returns `Err` if a session already exists.
    pub fn acquire_session(&self) -> Result<()> {
        if self
            .session_exists
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(Error::SessionAlreadyExists);
        }
        Ok(())
    }

    /// Releases the session, allowing another one to be acquired.
    pub fn release_session(&self) {
        self.session_exists.store(false, Ordering::Release);
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        self.release_session();
    }
}
