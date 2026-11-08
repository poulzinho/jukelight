//! The player monitor turns device snapshots into published player
//! snapshots and events (design decision 4).

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::time::Duration;

use futures::StreamExt;
use jukelight_application::PlayerMonitor;
use jukelight_application::testing::FakeDeviceMonitor;
use jukelight_domain::{
    Content, DeviceSnapshot, Origin, PlaybackState, PlayerEvent, Position, QueueView, Timestamp,
    Track,
};

fn device(title: &str, elapsed_s: u64, at_ms: u64) -> DeviceSnapshot {
    DeviceSnapshot {
        maker: None,
        name: "Example Player".into(),
        player_key: "aabbccddeeff".into(),
        online: true,
        state: PlaybackState::Playing,
        origin: Origin::Service {
            brand: None,
            id: "Qobuz".into(),
            name: "Qobuz".into(),
            cast: false,
        },
        content: Some(Content {
            track: Track {
                title: title.into(),
                artists: vec![],
                album: None,
            },
            artwork: None,
            position: Some(Position {
                elapsed: Duration::from_secs(elapsed_s),
                duration: Some(Duration::from_secs(300)),
                observed_at: Timestamp::from_millis(at_ms),
            }),
            quality: None,
            queue: QueueView::NotApplicable,
            service: None,
            palette: None,
        }),
    }
}

#[test]
fn device_snapshot_becomes_the_current_player_snapshot() {
    let monitor = PlayerMonitor::new(Vec::new());
    let handle = monitor.handle();

    monitor.apply(&device("Song A", 10, 0));

    let current = handle.current();
    assert_eq!(current.content.unwrap().track.title, "Song A");
    assert_eq!(current.session.label, "Qobuz");
}

#[test]
fn events_are_broadcast() {
    let monitor = PlayerMonitor::new(Vec::new());
    let mut events = monitor.handle().subscribe_events();

    monitor.apply(&device("Song A", 10, 0));
    monitor.apply(&device("Song B", 0, 1000));

    let received: Vec<_> = std::iter::from_fn(|| events.try_recv().ok()).collect();
    assert!(received.contains(&PlayerEvent::PlayerCameOnline));
    assert_eq!(
        received
            .iter()
            .filter(|e| **e == PlayerEvent::TrackChanged)
            .count(),
        2
    );
}

#[test]
fn published_snapshot_changes_only_on_material_changes() {
    let monitor = PlayerMonitor::new(Vec::new());
    let handle = monitor.handle();
    let mut published = handle.subscribe_published();

    monitor.apply(&device("Song A", 10, 0));
    assert!(published.has_changed().unwrap());
    published.mark_unchanged();

    // Normal progress: current snapshot moves, published one does not.
    monitor.apply(&device("Song A", 40, 30_000));
    assert!(!published.has_changed().unwrap());
    let current = handle.current();
    let elapsed = current.content.unwrap().position.unwrap().elapsed;
    assert_eq!(elapsed, Duration::from_secs(40));

    // A seek is material.
    monitor.apply(&device("Song A", 200, 31_000));
    assert!(published.has_changed().unwrap());
}

#[test]
fn late_subscribers_get_the_current_snapshot() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&device("Song A", 10, 0));

    let published = monitor.handle().subscribe_published();
    assert_eq!(
        published.borrow().content.as_ref().unwrap().track.title,
        "Song A"
    );
}

#[tokio::test]
async fn run_follows_the_device_monitor() {
    let (device_monitor, sender) = FakeDeviceMonitor::new();
    let monitor = PlayerMonitor::new(Vec::new());
    let handle = monitor.handle();
    let mut published = tokio_stream_like(handle.subscribe_published())
        .skip(1)
        .boxed();

    let task = tokio::spawn(monitor.run(std::sync::Arc::new(device_monitor)));
    sender.send(device("Song A", 10, 0)).unwrap();

    let snapshot = tokio::time::timeout(Duration::from_secs(1), published.next())
        .await
        .expect("a snapshot within a second")
        .unwrap();
    assert_eq!(snapshot.content.unwrap().track.title, "Song A");
    task.abort();
}

/// A watch receiver as a stream of values (initial value first).
fn tokio_stream_like(
    receiver: tokio::sync::watch::Receiver<jukelight_domain::PlayerSnapshot>,
) -> impl futures::Stream<Item = jukelight_domain::PlayerSnapshot> {
    futures::stream::unfold((receiver, true), |(mut receiver, first)| async move {
        if !first {
            receiver.changed().await.ok()?;
        }
        let value = receiver.borrow_and_update().clone();
        Some((value, (receiver, false)))
    })
}

// DEV-3: session-arbitration, "Provider updates refresh the snapshot".

mod provider_updates {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use futures::StreamExt;
    use futures::stream::BoxStream;
    use jukelight_application::testing::{FakeDeviceMonitor, FakeProvider};
    use jukelight_application::{PlayerMonitor, SessionProvider};
    use jukelight_domain::{
        Content, DeviceSnapshot, Origin, PlaybackState, PlayerSnapshot, QueueView, Track,
    };

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
            queue: QueueView::NotApplicable,
            service: None,
            palette: None,
        }
    }

    fn streaming_device(name: &str) -> DeviceSnapshot {
        DeviceSnapshot {
            maker: None,
            name: name.into(),
            player_key: "aabbccddeeff".into(),
            online: true,
            state: PlaybackState::Playing,
            origin: Origin::Stream {
                url: "http://192.168.1.50:8097/flow/s/aabbccddeeff/x.flac".into(),
            },
            content: Some(content("Music Assistant")),
        }
    }

    async fn next_title(
        published: &mut tokio::sync::watch::Receiver<PlayerSnapshot>,
        title: &str,
    ) -> bool {
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                let current = published
                    .borrow_and_update()
                    .content
                    .as_ref()
                    .map(|c| c.track.title.clone());
                if current.as_deref() == Some(title) {
                    return true;
                }
                if published.changed().await.is_err() {
                    return false;
                }
            }
        })
        .await
        .unwrap_or(false)
    }

    #[tokio::test]
    async fn track_change_seen_only_by_the_provider() {
        let provider = Arc::new(FakeProvider::claiming(
            "Example Provider",
            Some(content("Song A")),
        ));
        let (device_monitor, devices) = FakeDeviceMonitor::new();
        let monitor = PlayerMonitor::new(vec![provider.clone() as Arc<dyn SessionProvider>]);
        let mut published = monitor.handle().subscribe_published();
        let task = tokio::spawn(monitor.run(Arc::new(device_monitor)));

        devices.send(streaming_device("Example Player")).unwrap();
        assert!(next_title(&mut published, "Song A").await, "first snapshot");

        // The device reports nothing new; only the provider's content changes.
        provider.set_content(Some(content("Song B")));
        assert!(
            next_title(&mut published, "Song B").await,
            "provider update"
        );
        task.abort();
    }

    /// Titles its content after the last device report it was given.
    struct EchoProvider {
        followed: Mutex<Option<String>>,
    }

    impl SessionProvider for EchoProvider {
        fn label(&self) -> &'static str {
            "Echo"
        }
        fn claims(&self, _device: &DeviceSnapshot) -> bool {
            true
        }
        fn content(&self) -> Option<Content> {
            self.followed.lock().unwrap().as_deref().map(content)
        }
        fn follow(&self, device: &DeviceSnapshot) {
            *self.followed.lock().unwrap() = Some(device.name.clone());
        }
        fn updates(&self) -> BoxStream<'static, ()> {
            futures::stream::pending().boxed()
        }
    }

    #[test]
    fn provider_is_told_about_the_device_before_the_rebuild() {
        let provider = Arc::new(EchoProvider {
            followed: Mutex::new(None),
        });
        let monitor = PlayerMonitor::new(vec![provider as Arc<dyn SessionProvider>]);

        monitor.apply(&streaming_device("First Report"));
        assert_eq!(
            monitor.handle().current().content.unwrap().track.title,
            "First Report"
        );
        monitor.apply(&streaming_device("Second Report"));
        assert_eq!(
            monitor.handle().current().content.unwrap().track.title,
            "Second Report"
        );
    }
}
