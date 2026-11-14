//! Spec: session-arbitration, "A new session settles before it shows".

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::stream::BoxStream;
use jukelight_application::testing::{FakeDeviceMonitor, ticks};
use jukelight_application::{PlayerMonitor, SessionProvider};
use jukelight_domain::{
    Brand, Content, DeviceSnapshot, Origin, PlaybackState, PlayerSnapshot, QueueView, Track,
};
use tokio::sync::{mpsc, watch};

fn content(title: &str) -> Content {
    Content {
        track: Track {
            title: title.into(),
            artists: vec![],
            album: None,
        },
        artwork: None,
        position: None,
        quality: None,
        delivery: None,
        queue: QueueView::NotApplicable,
        service: None,
        palette: None,
        speakers: Vec::new(),
        held: false,
    }
}

fn device(state: PlaybackState, origin: Origin, title: &str) -> DeviceSnapshot {
    DeviceSnapshot {
        maker: None,
        name: "Example Player".into(),
        player_key: "aabbccddeeff".into(),
        online: true,
        state,
        origin,
        content: Some(content(title)),
        outputs: Vec::new(),
    }
}

fn airplay() -> Origin {
    Origin::Service {
        brand: Brand::new("airplay"),
        id: "AirPlay".into(),
        name: "AirPlay".into(),
        cast: true,
    }
}

fn qobuz() -> Origin {
    Origin::Service {
        brand: None,
        id: "Qobuz".into(),
        name: "Qobuz".into(),
        cast: false,
    }
}

/// A provider that answers when the test says so: until then it is
/// settling AirPlay origins.
struct AnsweringProvider {
    state: Mutex<(bool, Option<Content>)>,
    updates: watch::Sender<()>,
}

impl AnsweringProvider {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new((false, None)),
            updates: watch::channel(()).0,
        })
    }

    /// Music Assistant answers: it drives the player, with this content.
    fn answer(&self, title: &str) {
        *self.state.lock().unwrap() = (true, Some(content(title)));
        self.updates.send_replace(());
    }

    fn answered(&self) -> bool {
        self.state.lock().unwrap().0
    }
}

impl SessionProvider for AnsweringProvider {
    fn label(&self) -> &'static str {
        "Example Provider"
    }
    fn claims(&self, device: &DeviceSnapshot) -> bool {
        self.state.lock().unwrap().1.is_some()
            && matches!(device.origin, Origin::Service { ref id, .. } if id == "AirPlay")
    }
    fn content(&self) -> Option<Content> {
        self.state.lock().unwrap().1.clone()
    }
    fn follow(&self, _device: &DeviceSnapshot) {}
    fn settling(&self, device: &DeviceSnapshot) -> bool {
        !self.answered()
            && matches!(device.origin, Origin::Service { ref id, .. } if id == "AirPlay")
    }
    fn updates(&self) -> BoxStream<'static, ()> {
        ticks(&self.updates)
    }
}

/// Runs a monitor; returns the device sender and every published title, in order.
fn start(
    provider: Arc<AnsweringProvider>,
) -> (
    mpsc::UnboundedSender<DeviceSnapshot>,
    Arc<Mutex<Vec<String>>>,
) {
    let (device_monitor, devices) = FakeDeviceMonitor::new();
    let monitor = PlayerMonitor::new(vec![provider as Arc<dyn SessionProvider>]);
    let mut published = monitor.handle().subscribe_published();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = Arc::clone(&seen);
    tokio::spawn(async move {
        while published.changed().await.is_ok() {
            let snapshot: PlayerSnapshot = published.borrow_and_update().clone();
            let title = match (snapshot.state, snapshot.content) {
                (PlaybackState::Stopped, _) => "idle".to_owned(),
                (_, Some(content)) => content.track.title,
                (_, None) => "nothing".to_owned(),
            };
            record.lock().unwrap().push(title);
        }
    });
    tokio::spawn(monitor.run(Arc::new(device_monitor)));
    (devices, seen)
}

async fn settle() {
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
}

#[tokio::test(start_paused = true)]
async fn a_group_resumes() {
    let provider = AnsweringProvider::new();
    let (devices, seen) = start(Arc::clone(&provider));
    devices
        .send(device(PlaybackState::Stopped, qobuz(), "Example Song"))
        .unwrap();
    settle().await;

    // The group starts: the player reports AirPlay before Music Assistant answers.
    devices
        .send(device(PlaybackState::Playing, airplay(), "AirPlay"))
        .unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    provider.answer("Group Song");
    settle().await;

    assert_eq!(*seen.lock().unwrap(), ["idle", "Group Song"]);
}

#[tokio::test(start_paused = true)]
async fn the_provider_doesnt_answer_in_time() {
    let provider = AnsweringProvider::new();
    let (devices, seen) = start(provider);
    devices
        .send(device(PlaybackState::Stopped, qobuz(), "Example Song"))
        .unwrap();
    settle().await;

    devices
        .send(device(PlaybackState::Playing, airplay(), "AirPlay"))
        .unwrap();
    tokio::time::sleep(Duration::from_millis(1_900)).await;
    settle().await;
    assert_eq!(*seen.lock().unwrap(), ["idle"], "still settling");

    tokio::time::sleep(Duration::from_millis(200)).await;
    settle().await;
    assert_eq!(*seen.lock().unwrap(), ["idle", "AirPlay"]);
}

#[tokio::test(start_paused = true)]
async fn nothing_settling() {
    let provider = AnsweringProvider::new();
    let (devices, seen) = start(provider);
    devices
        .send(device(PlaybackState::Stopped, qobuz(), "Example Song"))
        .unwrap();
    settle().await;

    // Qobuz plays on the player: no provider is settling it.
    devices
        .send(device(PlaybackState::Playing, qobuz(), "Another Song"))
        .unwrap();
    settle().await;
    assert_eq!(*seen.lock().unwrap(), ["idle", "Another Song"]);
}
