//! The shared session provider contract, run against the fake provider.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use jukelight_application::testing::FakeProvider;
use jukelight_application::testing::contract::{ProviderHarness, session_provider_contract};
use jukelight_domain::{Content, DeviceSnapshot, Origin, PlaybackState, QueueView, Track};

fn content(title: String) -> Content {
    Content {
        track: Track {
            title,
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

fn device(origin: Origin) -> DeviceSnapshot {
    DeviceSnapshot {
        maker: None,
        name: "Example Player".into(),
        player_key: "aabbccddeeff".into(),
        online: true,
        state: PlaybackState::Playing,
        origin,
        content: None,
    }
}

/// Drives a fake provider: "unreachable" clears its content.
struct Harness {
    provider: Arc<FakeProvider>,
    version: AtomicU32,
}

impl Harness {
    fn current(&self) -> Content {
        content(format!("Song {}", self.version.load(Ordering::SeqCst)))
    }
}

impl ProviderHarness for Harness {
    fn claimed_device(&self) -> DeviceSnapshot {
        device(Origin::Stream {
            url: "http://192.168.1.50:8097/flow/s/aabbccddeeff/x.flac".into(),
        })
    }
    fn unclaimed_device(&self) -> DeviceSnapshot {
        device(Origin::Service {
            brand: None,
            id: "Qobuz".into(),
            name: "Qobuz".into(),
            cast: false,
        })
    }
    fn make_unreachable(&self) {
        self.provider.set_content(None);
    }
    fn make_reachable(&self) {
        self.provider.set_content(Some(self.current()));
    }
    fn change_content(&self) {
        self.version.fetch_add(1, Ordering::SeqCst);
        self.provider.set_content(Some(self.current()));
    }
}

#[tokio::test]
async fn fake_provider_meets_the_contract() {
    let provider = Arc::new(FakeProvider::claiming("Example Provider", None));
    let harness = Harness {
        provider: Arc::clone(&provider),
        version: AtomicU32::new(0),
    };
    session_provider_contract(provider.as_ref(), &harness).await;
    assert!(
        !provider.followed().is_empty(),
        "the contract follows the claimed device"
    );
}
