//! Spec: session-arbitration.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::sync::Arc;

use jukelight_application::testing::FakeProvider;
use jukelight_application::{SessionProvider, arbitrate};
use jukelight_domain::{
    AudioInput, Content, DeviceSnapshot, InputKind, Origin, PlaybackState, QueueView, Timestamp,
    Track,
};

fn content(title: &str) -> Content {
    Content {
        track: Track {
            title: title.into(),
            artists: vec!["Example Artist".into()],
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

fn device(origin: Origin, state: PlaybackState, title: &str) -> DeviceSnapshot {
    DeviceSnapshot {
        name: "Example Player".into(),
        player_key: "aabbccddeeff".into(),
        online: true,
        state,
        origin,
        content: Some(content(title)),
    }
}

fn stream() -> Origin {
    Origin::Stream {
        url: "http://192.168.1.50:8097/flow/abc/aabbccddeeff/def.flac".into(),
    }
}

fn providers(list: Vec<FakeProvider>) -> Vec<Arc<dyn SessionProvider>> {
    list.into_iter()
        .map(|p| Arc::new(p) as Arc<dyn SessionProvider>)
        .collect()
}

#[test]
fn provider_drives_the_device() {
    let provider = FakeProvider::claiming("Example Provider", Some(content("Provider Song")));
    let snapshot = arbitrate(
        &device(stream(), PlaybackState::Playing, "Example Provider"),
        &providers(vec![provider]),
    );

    assert_eq!(snapshot.content.unwrap().track.title, "Provider Song");
    assert_eq!(snapshot.session.label, "Example Provider");
    assert!(snapshot.session.available);
}

#[test]
fn state_comes_from_the_device() {
    let provider = FakeProvider::claiming("Example Provider", Some(content("Provider Song")));
    let snapshot = arbitrate(
        &device(stream(), PlaybackState::Paused, "x"),
        &providers(vec![provider]),
    );

    assert_eq!(snapshot.state, PlaybackState::Paused);
    assert!(snapshot.online);
    assert_eq!(snapshot.player_name, "Example Player");
}

#[test]
fn provider_down() {
    let provider = FakeProvider::claiming("Example Provider", None);
    let snapshot = arbitrate(
        &device(stream(), PlaybackState::Playing, "x"),
        &providers(vec![provider]),
    );

    assert!(!snapshot.session.available);
    let shown = snapshot.content.unwrap();
    assert_eq!(shown.track.title, "Playing from Example Provider");
    assert_eq!(shown.track.artists, Vec::<String>::new());
    assert_eq!(shown.track.album, None);
    assert_eq!(shown.artwork, None);
    assert_eq!(shown.position, None);
    assert_eq!(shown.queue, QueueView::NotApplicable);
}

#[test]
fn native_service() {
    let origin = Origin::Service {
        id: "Qobuz".into(),
        name: "Qobuz".into(),
        cast: false,
    };
    let snapshot = arbitrate(&device(origin, PlaybackState::Playing, "Example Song"), &[]);

    assert_eq!(snapshot.content.unwrap().track.title, "Example Song");
    assert_eq!(snapshot.session.label, "Qobuz");
    assert!(snapshot.session.available);
}

#[test]
fn unclaimed_stream() {
    let snapshot = arbitrate(
        &device(stream(), PlaybackState::Playing, "Music Assistant"),
        &[],
    );

    assert_eq!(snapshot.content.unwrap().track.title, "Music Assistant");
    assert_eq!(snapshot.session.label, "Stream");
}

#[test]
fn provider_that_does_not_claim_is_ignored() {
    let provider = FakeProvider::not_claiming("Example Provider");
    let snapshot = arbitrate(
        &device(stream(), PlaybackState::Playing, "Device Song"),
        &providers(vec![provider]),
    );

    assert_eq!(snapshot.content.unwrap().track.title, "Device Song");
    assert_eq!(snapshot.session.label, "Stream");
}

#[test]
fn no_origin_has_an_empty_label() {
    let snapshot = arbitrate(&device(Origin::None, PlaybackState::Stopped, "x"), &[]);
    assert_eq!(snapshot.session.label, "");
}

#[test]
fn claim_while_the_provider_is_down() {
    // A provider decides its claim from configuration and origin alone; its
    // content being unavailable does not change the claim.
    let provider = FakeProvider::claiming("Example Provider", None);
    let device = device(stream(), PlaybackState::Playing, "x");

    assert!(provider.claims(&device));
    assert_eq!(
        arbitrate(&device, &providers(vec![provider])).session.label,
        "Example Provider"
    );
}

#[test]
fn provider_names_the_music_service() {
    let mut named = content("Provider Song");
    named.service = Some("Example Service".into());
    let provider = FakeProvider::claiming("Example Provider", Some(named));
    let snapshot = arbitrate(
        &device(stream(), PlaybackState::Playing, "x"),
        &providers(vec![provider]),
    );

    assert_eq!(snapshot.session.label, "Example Provider · Example Service");
}

// An input origin: its kind as the label, the input on the snapshot, no track.

fn input_device(kind: InputKind) -> DeviceSnapshot {
    DeviceSnapshot {
        content: None,
        ..device(
            Origin::Input(AudioInput {
                kind,
                name: "Example Input".into(),
                selected_at: Timestamp::from_millis(1_000_000),
            }),
            PlaybackState::Playing,
            "unused",
        )
    }
}

#[test]
fn optical_input() {
    let snapshot = arbitrate(&input_device(InputKind::Optical), &[]);
    assert_eq!(snapshot.session.label, "Optical input");
    assert!(snapshot.session.available);
    let input = snapshot.input.expect("the input");
    assert_eq!(input.name, "Example Input");
    assert_eq!(input.kind, InputKind::Optical);
    assert_eq!(snapshot.content, None);
}

#[test]
fn input_kind_labels() {
    for (kind, label) in [
        (InputKind::Analog, "Analog input"),
        (InputKind::HdmiArc, "HDMI ARC"),
        (InputKind::Other, "Input"),
    ] {
        assert_eq!(arbitrate(&input_device(kind), &[]).session.label, label);
    }
}

#[test]
fn other_origins_have_no_input() {
    let snapshot = arbitrate(&device(stream(), PlaybackState::Playing, "Song"), &[]);
    assert_eq!(snapshot.input, None);
}
