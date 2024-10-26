//! This module provides a configuration setup for interacting with the Tradier API, 
//! including REST and WebSocket streaming configurations. The module structures credentials, 
//! REST API settings, and streaming settings into distinct configurations. It also provides 
//! utility functions to load environment variables with defaults and a set of tests to 
//! validate configuration behavior.
mod base;

pub use base::{Config, Credentials, RestApiConfig, StreamingConfig, get_env_or_default};