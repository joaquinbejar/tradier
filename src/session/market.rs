use crate::config::base::Config;
use crate::session::session::{Session, SessionType};
use std::error::Error;

pub struct MarketSession(Session);

impl MarketSession {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        match Session::new(SessionType::Market, config).await {
            Ok(session) => Ok(MarketSession(session)),
            Err(e) => {
                Err(format!("Error creating account session: {}", e).into())
            }
        }
    }

    pub fn get_session_id(&self) -> &str {
        self.0.get_session_id()
    }

    pub fn get_websocket_url(&self) -> &str {
        self.0.get_websocket_url()
    }
}