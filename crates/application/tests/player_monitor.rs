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
        name: "Example Player".into(),
        player_key: "aabbccddeeff".into(),
        online: true,
        state: PlaybackState::Playing,
        origin: Origin::Service {
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
