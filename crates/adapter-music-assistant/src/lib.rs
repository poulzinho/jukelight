//! Music Assistant session provider: recognises Music Assistant streams on
//! the player and follows its playback through Music Assistant's HTTP API.

pub mod api;
mod claim;
mod mapping;
mod provider;

pub use claim::{ClaimRule, UrlError};
pub use mapping::{Responses, UP_NEXT, to_content};
pub use provider::{MusicAssistantConfig, MusicAssistantProvider, Timing, Token};
