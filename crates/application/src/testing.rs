//! In-memory implementations of the ports, and the shared port contract
//! suites, for tests in this and other crates.

pub mod contract;

use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use futures::stream::BoxStream;
use futures::{StreamExt, stream};
use jukelight_domain::{Brand, Content, DeviceSnapshot, Origin, Timestamp};
use tokio::sync::{mpsc, watch};

use crate::{Clock, DeviceMonitor, SessionProvider};

/// A provider whose claim and content are set by the test. Setting the
/// content publishes an update; the device reports it follows are recorded.
pub struct FakeProvider {
    label: String,
    brand: Option<Brand>,
    claims: bool,
    content: Mutex<Option<Content>>,
    followed: Mutex<Vec<DeviceSnapshot>>,
    updates: watch::Sender<()>,
}

impl FakeProvider {
    pub fn claiming(label: &str, content: Option<Content>) -> Self {
        Self {
            label: label.into(),
            brand: None,
            claims: true,
            content: Mutex::new(content),
            followed: Mutex::new(Vec::new()),
            updates: watch::channel(()).0,
        }
    }

    pub fn not_claiming(label: &str) -> Self {
        Self {
            label: label.into(),
            brand: None,
            claims: false,
            content: Mutex::new(None),
            followed: Mutex::new(Vec::new()),
            updates: watch::channel(()).0,
        }
    }

    /// The same provider, with a brand.
    #[must_use]
    pub fn with_brand(mut self, brand: &str) -> Self {
        self.brand = Brand::new(brand);
        self
    }

    pub fn set_content(&self, content: Option<Content>) {
        *lock(&self.content) = content;
        self.updates.send_replace(());
    }

    /// The device reports this provider was told about, oldest first.
    pub fn followed(&self) -> Vec<DeviceSnapshot> {
        lock(&self.followed).clone()
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Ticks whenever `sender` publishes; the stream ends with the sender.
pub fn ticks(sender: &watch::Sender<()>) -> BoxStream<'static, ()> {
    stream::unfold(sender.subscribe(), |mut receiver| async move {
        receiver.changed().await.ok().map(|()| ((), receiver))
    })
    .boxed()
}

impl SessionProvider for FakeProvider {
    fn label(&self) -> &str {
        &self.label
    }

    fn brand(&self) -> Option<Brand> {
        self.brand.clone()
    }

    /// A claiming fake claims stream origins, like a real provider would.
    fn claims(&self, device: &DeviceSnapshot) -> bool {
        self.claims && matches!(device.origin, Origin::Stream { .. })
    }

    fn content(&self) -> Option<Content> {
        lock(&self.content).clone()
    }

    fn follow(&self, device: &DeviceSnapshot) {
        lock(&self.followed).push(device.clone());
    }

    fn updates(&self) -> BoxStream<'static, ()> {
        ticks(&self.updates)
    }
}

/// A device monitor fed by the test through a channel.
pub struct FakeDeviceMonitor {
    receiver: Mutex<Option<mpsc::UnboundedReceiver<DeviceSnapshot>>>,
}

impl FakeDeviceMonitor {
    pub fn new() -> (Self, mpsc::UnboundedSender<DeviceSnapshot>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        (
            Self {
                receiver: Mutex::new(Some(receiver)),
            },
            sender,
        )
    }
}

impl DeviceMonitor for FakeDeviceMonitor {
    fn snapshots(&self) -> BoxStream<'static, DeviceSnapshot> {
        let receiver = self
            .receiver
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        match receiver {
            Some(receiver) => stream::unfold(receiver, |mut receiver| async move {
                receiver.recv().await.map(|snapshot| (snapshot, receiver))
            })
            .boxed(),
            None => stream::empty().boxed(),
        }
    }
}

/// A clock that only moves when the test moves it.
#[derive(Default)]
pub struct FixedClock(AtomicU64);

impl FixedClock {
    pub fn at(millis: u64) -> Self {
        Self(AtomicU64::new(millis))
    }

    pub fn set(&self, millis: u64) {
        self.0.store(millis, Ordering::SeqCst);
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_millis(self.0.load(Ordering::SeqCst))
    }
}
