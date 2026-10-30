//! Jukelight's application layer: the ports adapters implement, and the
//! services that turn device reports into the snapshot the kiosk shows.

mod arbiter;
mod monitor;
mod ports;
pub mod testing;

pub use arbiter::arbitrate;
pub use monitor::{PlayerHandle, PlayerMonitor};
pub use ports::{
    ArtworkError, ArtworkId, ArtworkSource, Clock, DeviceMonitor, Image, SessionProvider,
    SystemClock,
};
