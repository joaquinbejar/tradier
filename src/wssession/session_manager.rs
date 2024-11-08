use crate::{Error, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

/// Manages the existence of a single session within a system.
///
/// `SessionManager` is an internal utility designed to enforce a single active session at any
/// given time, ensuring that only one `Session` is accessible across the system. This is
/// particularly useful when only one connection to the service, such as Tradier, should be active.
/// Users of this library should interact with `Session`, which will handle session management
/// through `SessionManager` internally.
///
/// ⚠️ **Note:** Only one `SessionManager` instance should exist within the system at a time.
/// For applications requiring a single active session manager globally, use `global_session_manager`.
/// In single-threaded environments, a non-global instance may be used directly within the runtime.
///
/// # Examples
///
/// ## Using `SessionManager` as a Global Singleton
///
/// In multi-threaded applications where only one active session should exist across all threads,
/// `global_session_manager` provides a singleton `SessionManager` instance that can coordinate
/// session access throughout the application. Typically, this is accessed indirectly through `Session`.
///
/// ```rust
/// use tradier::wssession::global_session_manager;
///
/// // Obtain the global `SessionManager` instance
/// let manager = global_session_manager();
///
/// // Normally, `Session` interacts with `SessionManager` internally
/// // to manage session state, so users do not need to directly invoke its methods.
/// ```
///
/// ## Using `SessionManager` in a Single-Threaded Runtime
///
/// If the application runs in a single-threaded runtime (e.g., a Tokio `LocalSet`), it is safe
/// to create a local instance of `SessionManager` rather than a global one. This approach is efficient
/// as single-threaded execution ensures only one instance will access the session state.
///
/// ```rust
/// use tradier::wssession::SessionManager;
/// use tokio::task::LocalSet;
///
/// #[tokio::main]
/// async fn main() {
///     let local = LocalSet::new();
///     let session_manager = SessionManager::default(); // Local instance
///
///     local.run_until(async move {
///         // `Session` would manage session acquisition internally
///     }).await;
/// }
/// ```
#[derive(Default, Debug)]
pub struct SessionManager {
    session_exists: AtomicBool,
}

impl SessionManager {
    // Internally acquires a session, enforcing a single active session.
    //
    // Returns `Ok(())` if the session was successfully acquired, or `Err(Error::SessionAlreadyExists)`
    // if a session is already active. This method is used by `Session` to manage access to the session
    // and is not intended to be called directly by end users.
    pub(crate) fn acquire_session(&self) -> Result<()> {
        if self
            .session_exists
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(Error::SessionAlreadyExists);
        }
        Ok(())
    }

    // Internally releases the session, allowing a new session to be acquired.
    //
    // This is called by `Session` to manage the session lifecycle, and users should not
    // need to call this method directly.
    pub(crate) fn release_session(&self) {
        self.session_exists.store(false, Ordering::Release);
    }
}

impl Drop for SessionManager {
    /// Automatically releases the session when the `SessionManager` is dropped.
    ///
    /// Ensures that any acquired session is released when a `SessionManager` instance goes
    /// out of scope, cleaning up session state. This is part of the internal session lifecycle
    /// management and is not intended for direct interaction by users.
    fn drop(&mut self) {
        self.release_session();
    }
}

/// Provides a globally accessible singleton instance of `SessionManager`.
///
/// This function ensures that only one instance of `SessionManager` exists across the
/// application. Use this function for managing session state in multi-threaded environments,
/// though users will typically interact with `Session`, which will handle the session
/// management through `SessionManager` internally.
///
/// # Example
///
/// ```rust
/// use tradier::wssession::global_session_manager;
///
/// let manager = global_session_manager();
/// // Normally, users interact with `Session`, which coordinates with `SessionManager`.
/// ```
pub fn global_session_manager() -> &'static SessionManager {
    static INSTANCE: OnceLock<SessionManager> = OnceLock::new();
    INSTANCE.get_or_init(SessionManager::default)
}
