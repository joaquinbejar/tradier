use crate::config::Config;
use crate::wssession::session::{Session, SessionType};
use crate::Result;

use super::session_manager::SessionManager;

/// `AccountSession` is a wrapper around a `Session` specifically for account-level WebSocket interactions.
/// This struct provides methods to initialize a new account session and retrieve essential session details.
///
/// `AccountSession` encapsulates `Session` with `SessionType::Account` and provides
/// convenience methods to interact with Tradier's account-level WebSocket services.
#[derive(Debug, Clone)]
pub struct AccountSession<'a>(Session<'a>);

impl<'a> AccountSession<'a> {
    /// Creates a new `AccountSession` using the provided configuration.
    ///
    /// # Arguments
    /// - `config`: A reference to a `Config` instance, which provides necessary details like API credentials.
    ///
    /// # Returns
    /// - `Ok(AccountSession)`: A new `AccountSession` instance if session creation is successful.
    /// - `Err(Box<dyn Error>)`: If session creation fails, an error is returned with details.
    ///
    /// ```should_panic
    /// use tradier::config::Config;
    /// use tradier::wssession::{AccountSession, SessionManager};
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = Config::new();
    ///     let session_manager = SessionManager::default();
    ///     let account_session = AccountSession::new(&session_manager, &config)
    ///         .await
    ///         .expect("caller to handle failure");
    /// }
    /// ```
    pub async fn new(session_manager: &'a SessionManager, config: &Config) -> Result<Self> {
        Ok(AccountSession(
            Session::new(session_manager, SessionType::Account, config).await?,
        ))
    }

    /// Retrieves the session ID associated with this account session.
    ///
    /// # Returns
    /// - `&str`: The session ID as a string reference, uniquely identifying this session.
    ///
    /// # Example
    /// ```should_panic
    /// use tradier::config::Config;
    /// use tradier::wssession::{AccountSession, SessionManager};
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = Config::new();
    ///     let session_manager = SessionManager::default();
    ///     let account_session = AccountSession::new(&session_manager, &config)
    ///         .await
    ///         .expect("caller to handle failure");
    ///     let session_id = account_session.get_session_id();
    ///     println!("Session ID: {}", session_id);
    /// }
    /// ```
    pub fn get_session_id(&self) -> &str {
        self.0.get_session_id()
    }

    /// Retrieves the WebSocket URL associated with this account session.
    ///
    /// # Returns
    /// - `&str`: The WebSocket URL as a string reference, used to establish a connection for account data.
    ///
    /// # Example
    /// ```should_panic
    /// use tradier::config::Config;
    /// use tradier::wssession::{SessionManager, AccountSession};
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = Config::new();
    ///     let session_manager = SessionManager::default();
    ///     let account_session = AccountSession::new(&session_manager, &config)
    ///         .await
    ///         .expect("caller to handle failure");
    ///     let websocket_url = account_session.get_websocket_url();
    ///     println!("WebSocket URL: {}", websocket_url);
    /// }
    /// ```

    pub fn get_websocket_url(&self) -> &str {
        self.0.get_websocket_url()
    }
}
