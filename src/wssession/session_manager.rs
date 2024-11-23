use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;

use crate::Error;
use crate::Result;

/// Manages the existence of a single session within a system.
///
/// `SessionManager` is an internal utility that enforces a single active session
/// at any given time. It is primarily used internally by higher-level abstractions like [`Session`]
/// to coordinate access to WebSocket sessions or other shared resources.
///
/// ### Key Features
/// - **Thread-Safe**: Uses atomic operations to safely manage session state in multi-threaded environments.
/// - **Lifecycle Management**: Ensures proper acquisition and release of session state.
/// - **Internal Use Only**: This type is not exposed outside of the crate. Use [`Session`] or
///   other public abstractions for interacting with session-related functionality.
///
/// ### Warning
/// - Only one `SessionManager` instance should exist in the system. Use the
///   [`GLOBAL_SESSION_MANAGER`] for global access in multi-threaded contexts.
///
/// ### Internal Use
/// Developers working within this crate can use `SessionManager` to manage session lifecycle
/// explicitly. For most external users, session management is handled transparently by [`Session`].
///
/// # Examples
///
/// ## Internal Use: Acquiring and Releasing a Session
///
/// The following example demonstrates how crate-internal code can acquire and release a session:
///
/// ```ignore
/// use tradier::wssession::session_manager::SessionManager;
///
/// let manager = SessionManager::default();
/// assert!(manager.acquire_session().is_ok());
/// assert!(manager.acquire_session().is_err()); // Second acquisition fails
/// manager.release_session(); // Releases the session
/// assert!(manager.acquire_session().is_ok()); // Session can be reacquired
/// ```
#[derive(Default, Debug)]
pub(crate) struct SessionManager {
    session_exists: AtomicBool,
}

impl SessionManager {
    /// Attempts to acquire a session, ensuring only one active session at a time.
    ///
    /// This method atomically sets the session state to "active". If a session is already
    /// active, it returns an error indicating that the session cannot be acquired.
    ///
    /// This method is for internal use by higher-level abstractions like [`Session`].
    ///
    /// # Returns
    /// - `Ok(())` if the session was successfully acquired.
    /// - `Err(Error::SessionAlreadyExists)` if a session is already active.
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

    /// Releases the active session, allowing a new session to be acquired.
    ///
    /// This method atomically sets the session state to "inactive". It is for internal
    /// use by components like [`Session`] to manage session lifecycles.
    pub(crate) fn release_session(&self) {
        self.session_exists.store(false, Ordering::Release);
    }
}

impl Drop for SessionManager {
    /// Automatically releases the session when the `SessionManager` is dropped.
    ///
    /// This ensures that any active session is released when a `SessionManager` instance
    /// goes out of scope. This method is primarily for internal use.
    fn drop(&mut self) {
        self.release_session();
    }
}

/// A globally accessible, lazily-initialized singleton instance of `SessionManager`.
///
/// The `GLOBAL_SESSION_MANAGER` provides a single, thread-safe instance of `SessionManager`
/// for use within the crate. It is not directly accessible outside the crate due to its
/// `pub(crate)` visibility.
///
/// Most session-related operations should use higher-level abstractions like [`Session`].
///
/// ### Key Features
/// - **Lazy Initialization**: The `SessionManager` is created only when first accessed.
/// - **Thread-Safe**: Ensures safe, concurrent access across threads.
///
/// ### Internal Usage
///
/// Direct access to `GLOBAL_SESSION_MANAGER` is intended only for crate-internal use. External
/// consumers of the library should use public abstractions like [`Session`].
pub(crate) static GLOBAL_SESSION_MANAGER: LazyLock<SessionManager> =
    LazyLock::new(SessionManager::default);
