//! # Tradier Account WebSocket Module
//!
//! This module provides functionality to interact with the Tradier Account WebSocket API, enabling real-time
//! streaming of account events such as order status updates, executions, and balance changes.
//!
//! ## Overview
//!
//! Tradier’s Account WebSocket API is designed to deliver immediate updates on account activities to authorized
//! clients. This API is useful for applications that need real-time insights into account status, including:
//! - Order placement, cancellation, and execution events
//! - Updates on account balances and positions
//!
//! An `AccountSession` in this module wraps Tradier’s WebSocket session specifically for account-related events,
//! ensuring efficient management of the WebSocket connection and enabling clients to receive streaming data
//! continuously. The `SessionManager` is used to enforce a singleton pattern, allowing only one session at a time
//! to prevent overlapping WebSocket connections.
//!
//! ## Usage
//!
//! To set up an account WebSocket session, initialize a `Config` with the necessary API credentials, and create
//! a `SessionManager` to manage the session state. Then, instantiate an `AccountSession` to begin streaming
//! account events from Tradier.
//!
//! ### Example
//!
//! ```no_run
//! use tradier::config::Config;
//! use tradier::wssession::AccountSession;
//! use tracing::info;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Load configuration with API credentials
//!     let config = Config::new();
//!     
//!     // Establish an account session
//!     match AccountSession::new(&config).await {
//!         Ok(account_session) => {
//!             info!("Account WebSocket session established with ID: {}", account_session.get_session_id());
//!             
//!             // Use `account_session` to start receiving account updates
//!             let websocket_url = account_session.get_websocket_url();
//!             println!("Connect to WebSocket at: {}", websocket_url);
//!         }
//!         Err(e) => {
//!             eprintln!("Failed to create account WebSocket session: {}", e);
//!         }
//!     }
//! }
//! ```
//!
//! ## Error Handling
//!
//! The `AccountSession` creation will return an error in cases such as:
//! - Missing or invalid API credentials
//! - Network connectivity issues preventing WebSocket connection establishment
//! - Attempting to create multiple sessions simultaneously, which is restricted by `SessionManager`
//!
//! For additional details on the API, refer to the [Tradier Account WebSocket documentation](https://documentation.tradier.com/brokerage-api/streaming/wss-account-websocket).

use crate::config::Config;
use crate::wssession::session::{Session, SessionType};
use crate::Result;

use super::global_session_manager;

/// `AccountSession` is a wrapper around a `Session` specifically for account-level WebSocket interactions.
///
/// This struct is designed to interact with Tradier's Account WebSocket API, which provides real-time
/// updates on account activities such as order status changes and account balance modifications. It
/// encapsulates a `Session` initialized with `SessionType::Account`, offering convenient methods to
/// establish and manage WebSocket connections for account-related data.
///
/// For more details on the Tradier Account WebSocket API, see the official [documentation](https://documentation.tradier.com/brokerage-api/streaming/wss-account-websocket).
#[derive(Debug, Clone)]
pub struct AccountSession<'a>(Session<'a>);

impl<'a> AccountSession<'a> {
    /// Creates a new `AccountSession` using the provided configuration and session manager.
    ///
    /// This method establishes a WebSocket session that streams real-time account events from Tradier.
    /// Events may include updates to order statuses, executions, and account balances. Tradier requires
    /// an active session ID and authentication details in `Config` to access the WebSocket API.
    ///
    /// # Arguments
    ///
    /// * `session_manager` - A reference to `SessionManager`, ensuring only one session is active.
    /// * `config` - A reference to `Config`, which provides essential details such as API credentials.
    ///
    /// # Returns
    ///
    /// * `Ok(AccountSession)` - A new instance of `AccountSession` if the session creation is successful.
    /// * `Err(Box<dyn Error>)` - An error if session creation fails, e.g., due to network issues or invalid credentials.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tradier::config::Config;
    /// use tradier::wssession::AccountSession;
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = Config::new();
    ///     let account_session = AccountSession::new(&config)
    ///         .await
    ///         .expect("Failed to create account session");
    /// }
    /// ```
    pub async fn new(config: &Config) -> Result<Self> {
        Ok(AccountSession(
            Session::new(global_session_manager(), SessionType::Account, config).await?,
        ))
    }

    /// Retrieves the session ID associated with this account session.
    ///
    /// The session ID is a unique identifier provided by Tradier when a WebSocket connection is established.
    /// It is used to track and manage the session, especially useful for reconnecting or troubleshooting issues.
    ///
    /// # Returns
    ///
    /// * `&str` - The session ID as a string slice, uniquely identifying this session.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tradier::config::Config;
    /// use tradier::wssession::AccountSession;
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = Config::new();
    ///     let account_session = AccountSession::new(&config)
    ///         .await
    ///         .expect("Failed to create account session");
    ///     let session_id = account_session.get_session_id();
    ///     println!("Session ID: {}", session_id);
    /// }
    /// ```
    pub fn get_session_id(&self) -> &str {
        self.0.get_session_id()
    }

    /// Retrieves the WebSocket URL for this account session.
    ///
    /// This URL is used to establish a WebSocket connection with Tradier's account-level streaming service.
    /// The WebSocket connection provides real-time streaming data, allowing applications to receive live
    /// updates on account events.
    ///
    /// # Returns
    ///
    /// * `&str` - The WebSocket URL as a string slice, which can be used to initiate the WebSocket connection.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tradier::config::Config;
    /// use tradier::wssession::AccountSession;
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = Config::new();
    ///     let account_session = AccountSession::new(&config)
    ///         .await
    ///         .expect("Failed to create account session");
    ///     let websocket_url = account_session.get_websocket_url();
    ///     println!("WebSocket URL: {}", websocket_url);
    /// }
    /// ```
    pub fn get_websocket_url(&self) -> &str {
        self.0.get_websocket_url()
    }
}
