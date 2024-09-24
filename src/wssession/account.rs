use crate::config::base::Config;
use crate::wssession::session::{Session, SessionType};
use std::error::Error;

#[derive(Debug, Clone)]
pub struct AccountSession(Session);

impl AccountSession {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        match Session::new(SessionType::Account, config).await {
            Ok(session) => Ok(AccountSession(session)),
            Err(e) => Err(format!("Error creating account wssession: {}", e).into()),
        }
    }

    pub fn get_session_id(&self) -> &str {
        self.0.get_session_id()
    }

    pub fn get_websocket_url(&self) -> &str {
        self.0.get_websocket_url()
    }
}
