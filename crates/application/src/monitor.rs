use std::sync::{Arc, Mutex, PoisonError};

use futures::{StreamExt, stream};
use jukelight_domain::{DeviceSnapshot, PlayerEvent, PlayerSnapshot, detect_changes};
use tokio::sync::{broadcast, watch};

use crate::names::SpeakerNames;
use crate::{DeviceMonitor, SessionProvider, arbitrate};

const EVENT_CAPACITY: usize = 64;

/// Follows a device and keeps the player snapshot up to date.
///
/// Two snapshot channels exist: `current` changes with every report (it has
/// the freshest position), `published` only when something changed beyond
/// normal progress, which is what live clients need.
pub struct PlayerMonitor {
    providers: Vec<Arc<dyn SessionProvider>>,
    names: SpeakerNames,
    /// The last device report, to rebuild from when a provider updates.
    last_device: Mutex<Option<DeviceSnapshot>>,
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
        Self::with_names(providers, SpeakerNames::default())
    }

    /// With your names for speakers and the player's output speaker.
    pub fn with_names(providers: Vec<Arc<dyn SessionProvider>>, names: SpeakerNames) -> Self {
        let initial = PlayerSnapshot::nothing_playing("");
        let (current, _) = watch::channel(initial.clone());
        let (published, _) = watch::channel(initial);
        let (events, _) = broadcast::channel(EVENT_CAPACITY);
        Self {
            providers,
            names,
            last_device: Mutex::new(None),
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

    /// Processes one device snapshot: providers are told about it first, then
    /// the player snapshot is rebuilt.
    pub fn apply(&self, device: &DeviceSnapshot) {
        for provider in &self.providers {
            provider.follow(device);
        }
        *self
            .last_device
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(device.clone());
        self.rebuild(device);
    }

    /// Rebuilds the player snapshot from the last device report, after a
    /// provider published new content.
    pub fn refresh(&self) {
        let device = self
            .last_device
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let Some(device) = device {
            self.rebuild(&device);
        }
    }

    fn rebuild(&self, device: &DeviceSnapshot) {
        let next = self.names.apply(arbitrate(device, &self.providers));
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

    /// Follows the device and the providers until the device stream ends.
    pub async fn run(self, device: Arc<dyn DeviceMonitor>) {
        let mut snapshots = device.snapshots();
        let mut updates = stream::select_all(self.providers.iter().map(|p| p.updates()));
        loop {
            tokio::select! {
                snapshot = snapshots.next() => match snapshot {
                    Some(snapshot) => self.apply(&snapshot),
                    None => return,
                },
                Some(()) = updates.next() => self.refresh(),
            }
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
