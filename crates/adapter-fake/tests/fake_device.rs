//! The fake device: scripted scenarios for kiosk work and tests.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::sync::Arc;

use futures::StreamExt;
use jukelight_adapter_fake::{FakeControls, FakeDevice, SCENARIOS};
use jukelight_application::DeviceMonitor;
use jukelight_application::testing::FixedClock;
use jukelight_application::testing::contract::{DeviceHarness, device_monitor_contract};
use jukelight_domain::{Palette, PlaybackState, QueueView, Rgb};

struct Harness(FakeControls);

impl DeviceHarness for Harness {
    fn make_unreachable(&self) {
        self.0.set_reachable(false);
    }
    fn make_reachable(&self) {
        self.0.set_reachable(true);
    }
}

fn device(scenario: &str) -> (FakeDevice, FakeControls) {
    FakeDevice::new(scenario, Arc::new(FixedClock::at(1_000_000))).unwrap()
}

async fn first(scenario: &str) -> jukelight_domain::DeviceSnapshot {
    let (device, _controls) = device(scenario);
    device.snapshots().next().await.unwrap()
}

#[tokio::test]
async fn device_monitor_contract_holds() {
    let (device, controls) = device("playing-with-queue");
    device_monitor_contract(&device, &Harness(controls)).await;
}

#[test]
fn all_scenarios_exist() {
    assert_eq!(
        SCENARIOS,
        [
            "playing-with-queue",
            "cast-without-queue",
            "long-title",
            "no-artwork",
            "hi-res",
            "album-colours",
            "paused",
            "stopped",
            "offline",
        ]
    );
    for scenario in SCENARIOS {
        assert!(
            FakeDevice::new(scenario, Arc::new(FixedClock::at(0))).is_ok(),
            "{scenario}"
        );
    }
}

#[test]
fn unknown_scenario_is_an_error() {
    let error = FakeDevice::new("nope", Arc::new(FixedClock::at(0)))
        .err()
        .unwrap();
    assert!(
        error.to_string().contains("playing-with-queue"),
        "should list valid scenarios: {error}"
    );
}

#[tokio::test]
async fn playing_with_queue() {
    let snapshot = first("playing-with-queue").await;
    assert!(snapshot.online);
    assert_eq!(snapshot.state, PlaybackState::Playing);
    let content = snapshot.content.unwrap();
    assert!(
        content
            .artwork
            .unwrap()
            .source()
            .starts_with("data:image/svg+xml;base64,")
    );
    let QueueView::Available(entries) = content.queue else {
        panic!("queue expected")
    };
    assert_eq!(entries.len(), 5);
}

#[tokio::test]
async fn cast_without_queue() {
    let content = first("cast-without-queue").await.content.unwrap();
    assert_eq!(content.queue, QueueView::NotApplicable);
    assert_eq!(content.quality.unwrap().label(), "160 kbps");
}

#[tokio::test]
async fn long_title() {
    let content = first("long-title").await.content.unwrap();
    assert!(content.track.title.len() > 80);
}

#[tokio::test]
async fn no_artwork() {
    assert_eq!(first("no-artwork").await.content.unwrap().artwork, None);
}

#[tokio::test]
async fn album_colours() {
    assert_eq!(
        first("album-colours").await.content.unwrap().palette,
        Some(Palette {
            background: Rgb(73, 51, 38),
            accent: Rgb(209, 147, 108),
        })
    );
    assert_eq!(
        first("playing-with-queue").await.content.unwrap().palette,
        None
    );
}

#[tokio::test]
async fn hi_res() {
    assert!(
        first("hi-res")
            .await
            .content
            .unwrap()
            .quality
            .unwrap()
            .is_hi_res()
    );
}

#[tokio::test]
async fn paused_stopped_and_offline() {
    assert_eq!(first("paused").await.state, PlaybackState::Paused);
    assert_eq!(first("stopped").await.state, PlaybackState::Stopped);
    let offline = first("offline").await;
    assert!(!offline.online);
    assert!(offline.content.is_some(), "offline keeps the last content");
}

#[tokio::test]
async fn position_is_observed_now() {
    let content = first("playing-with-queue").await.content.unwrap();
    assert_eq!(content.position.unwrap().observed_at.as_millis(), 1_000_000);
}
