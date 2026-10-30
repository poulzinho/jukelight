use std::sync::Arc;

use futures::StreamExt;
use jukelight_domain::{DeviceSnapshot, PlayerEvent, PlayerSnapshot, detect_changes};
use tokio::sync::{broadcast, watch};

use crate::{DeviceMonitor, SessionProvider, arbitrate};

const EVENT_CAPACITY: usize = 64;

/// Follows a device and keeps the player snapshot up to date.
///
/// Two snapshot channels exist: `current` changes with every report (it has
/// the freshest position), `published` only when something changed beyond
/// normal progress, which is what live clients need.
pub struct PlayerMonitor {
    providers: Vec<Arc<dyn SessionProvider>>,
    current: watch::Sender<PlayerSnapshot>,
    published: watch::Sender<PlayerSnapshot>,
    events: broadcast::Sender<PlayerEvent>,
}

/// Read access to a running monitor, for the HTTP layer and other consumers.
#[derive(Clone)]
pub struct PlayerHandle {
    current: watch::Receiver<PlayerSnapshot>,
    published: watch::Receiver<PlayerSnapshot>,
    events: broadcast::Sender<PlayerEvent>,
}

impl PlayerMonitor {
    pub fn new(providers: Vec<Arc<dyn SessionProvider>>) -> Self {
        let initial = PlayerSnapshot::nothing_playing("");
        let (current, _) = watch::channel(initial.clone());
        let (published, _) = watch::channel(initial);
        let (events, _) = broadcast::channel(EVENT_CAPACITY);
        Self {
            providers,
            current,
            published,
            events,
        }
    }

    pub fn handle(&self) -> PlayerHandle {
        PlayerHandle {
            current: self.current.subscribe(),
            published: self.published.subscribe(),
            events: self.events.clone(),
        }
    }

    /// Processes one device snapshot.
    pub fn apply(&self, device: &DeviceSnapshot) {
        let next = arbitrate(device, &self.providers);
        let changes = detect_changes(&self.current.borrow(), &next);

        if changes.material {
            self.published.send_replace(next.clone());
        }
        self.current.send_replace(next);
        for event in changes.events {
            // No receivers is fine: events are for whoever listens.
            let _ = self.events.send(event);
        }
    }

    /// Follows the device until its stream ends.
    pub async fn run(self, device: Arc<dyn DeviceMonitor>) {
        let mut snapshots = device.snapshots();
        while let Some(snapshot) = snapshots.next().await {
            self.apply(&snapshot);
        }
    }
}

impl PlayerHandle {
    /// The latest snapshot, with the freshest position.
    pub fn current(&self) -> PlayerSnapshot {
        self.current.borrow().clone()
    }

    /// Snapshots that changed beyond normal progress.
    pub fn subscribe_published(&self) -> watch::Receiver<PlayerSnapshot> {
        self.published.clone()
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<PlayerEvent> {
        self.events.subscribe()
    }
}
