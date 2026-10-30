//! Device monitor for BluOS players (BluOS Custom Integration API).

mod mapping;
mod monitor;
pub mod parse;

pub use mapping::{is_native, to_device_snapshot};
pub use monitor::{BluosConfig, BluosMonitor, Timing};
