use crate::config::base::Config;
use crate::session::session::{Session, SessionType};
use std::error::Error;

pub struct AccountSession(Session);

impl AccountSession {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        Ok(AccountSession(Session::new(SessionType::Account, config).await?))
    }

    pub fn get_session_id(&self) -> &str {
        self.0.get_session_id()
    }

    pub fn get_websocket_url(&self) -> &str {
        self.0.get_websocket_url()
    }
}