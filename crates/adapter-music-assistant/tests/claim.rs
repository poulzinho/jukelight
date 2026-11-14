//! Spec: session-music-assistant, "Claiming the session".

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use jukelight_adapter_music_assistant::ClaimRule;
use jukelight_domain::{DeviceSnapshot, Origin, PlaybackState};

const MA_URL: &str = "http://192.168.1.50:8095";
const FLOW_URL: &str = "http://192.168.1.50:8097/flow/session1/aabbccddeeff/00000000000000000000000000000001/aabbccddeeff.flac";

fn device(origin: Origin) -> DeviceSnapshot {
    DeviceSnapshot {
        maker: None,
        name: "Living Room".into(),
        player_key: "aabbccddeeff".into(),
        online: true,
        state: PlaybackState::Playing,
        origin,
        content: None,
        outputs: Vec::new(),
    }
}

fn stream(url: &str) -> Origin {
    Origin::Stream { url: url.into() }
}

#[test]
fn stream_from_music_assistant_for_this_player() {
    let rule = ClaimRule::new(MA_URL).unwrap();
    assert!(rule.claims(&device(stream(FLOW_URL))));
}

#[test]
fn stream_for_another_player() {
    let rule = ClaimRule::new(MA_URL).unwrap();
    let url = FLOW_URL.replace("aabbccddeeff", "112233445566");
    assert!(!rule.claims(&device(stream(&url))));
}

#[test]
fn stream_from_another_host() {
    let rule = ClaimRule::new(MA_URL).unwrap();
    let url = "http://192.168.1.60:8097/flow/session1/aabbccddeeff/x.flac";
    assert!(!rule.claims(&device(stream(url))));
}

#[test]
fn service_session() {
    let rule = ClaimRule::new(MA_URL).unwrap();
    let origin = Origin::Service {
        brand: None,
        id: "Qobuz".into(),
        name: "Qobuz".into(),
        cast: false,
    };
    assert!(!rule.claims(&device(origin)));
}

#[test]
fn host_names_compare_without_case() {
    let rule = ClaimRule::new("http://Music-Assistant.local:8095/").unwrap();
    let url = "http://music-assistant.local:8097/flow/s/aabbccddeeff/x.flac";
    assert!(rule.claims(&device(stream(url))));
}

#[test]
fn player_key_must_be_a_whole_path_segment() {
    let rule = ClaimRule::new(MA_URL).unwrap();
    let url = "http://192.168.1.50:8097/flow/s/xaabbccddeeff/x.flac";
    assert!(!rule.claims(&device(stream(url))));
}

#[test]
fn no_player_key_claims_nothing() {
    let rule = ClaimRule::new(MA_URL).unwrap();
    let mut device = device(stream("http://192.168.1.50:8097/flow//x.flac"));
    device.player_key = String::new();
    assert!(!rule.claims(&device));
}

#[test]
fn music_assistant_url_must_be_http() {
    assert!(ClaimRule::new("ftp://192.168.1.50").is_err());
    assert!(ClaimRule::new("not a url").is_err());
}
