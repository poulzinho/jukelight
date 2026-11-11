//! Jukelight's application layer: the ports adapters implement, and the
//! services that turn device reports into the snapshot the kiosk shows.

mod arbiter;
mod display;
mod monitor;
mod names;
mod ports;
pub mod testing;

pub use arbiter::arbitrate;
pub use display::{DisplayController, DisplayHandle};
pub use monitor::{PlayerHandle, PlayerMonitor};
pub use names::SpeakerNames;
pub use ports::{
    ArtworkError, ArtworkId, ArtworkSize, ArtworkSource, BrandLogos, Clock, DeviceMonitor, Image,
    SessionProvider, SystemClock,
};
