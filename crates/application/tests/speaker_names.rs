//! Spec: session-arbitration, "Speakers and outputs on the snapshot" and
//! "Your names for speakers".

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::collections::HashMap;
use std::sync::Arc;

use jukelight_application::testing::FakeProvider;
use jukelight_application::{SessionProvider, SpeakerNames, arbitrate};
use jukelight_domain::{
    Content, DeviceSnapshot, Origin, Output, OutputKind, PlaybackState, QueueView, Speaker,
    SpeakerKind, Track,
};

fn speaker(name: &str) -> Speaker {
    Speaker {
        name: name.into(),
        kind: SpeakerKind::Speaker,
        playing: true,
        quality: None,
    }
}

fn group(speakers: Vec<Speaker>, subwoofer: bool) -> jukelight_domain::PlayerSnapshot {
    let content = Content {
        track: Track {
            title: "Example Song".into(),
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
        speakers,
        held: false,
    };
    let device = DeviceSnapshot {
        maker: None,
        name: "Example Player".into(),
        player_key: "aabbccddeeff".into(),
        online: true,
        state: PlaybackState::Playing,
        origin: Origin::Stream {
            url: "http://192.168.1.50:8097/flow/a/aabbccddeeff/b.flac".into(),
        },
        content: None,
        outputs: if subwoofer {
            vec![Output {
                name: None,
                kind: OutputKind::Subwoofer,
            }]
        } else {
            vec![]
        },
    };
    let provider: Arc<dyn SessionProvider> =
        Arc::new(FakeProvider::claiming("Music Assistant", Some(content)));
    arbitrate(&device, &[provider])
}

#[test]
fn group_with_an_output_speaker_and_a_subwoofer() {
    let names = SpeakerNames::new(HashMap::new(), Some("Example Speaker".into()));
    let snapshot = names.apply(group(vec![speaker("Kitchen"), speaker("Office")], true));
    assert_eq!(snapshot.content.unwrap().speakers.len(), 2);
    assert_eq!(
        snapshot.outputs,
        vec![
            Output {
                name: Some("Example Speaker".into()),
                kind: OutputKind::Speaker
            },
            Output {
                name: None,
                kind: OutputKind::Subwoofer
            },
        ]
    );
}

#[test]
fn nothing_configured() {
    let snapshot = SpeakerNames::default().apply(group(vec![], false));
    assert_eq!(snapshot.outputs, vec![]);
}

#[test]
fn renamed_speaker() {
    let names = SpeakerNames::new(
        HashMap::from([("Example Portable Speaker".to_owned(), "Kitchen".to_owned())]),
        None,
    );
    let snapshot = names.apply(group(vec![speaker("Example Portable Speaker")], false));
    assert_eq!(snapshot.content.unwrap().speakers[0].name, "Kitchen");
}

#[test]
fn not_renamed() {
    let names = SpeakerNames::new(
        HashMap::from([("Other".to_owned(), "Kitchen".to_owned())]),
        None,
    );
    let snapshot = names.apply(group(vec![speaker("Example Web Player")], false));
    assert_eq!(
        snapshot.content.unwrap().speakers[0].name,
        "Example Web Player"
    );
}
