//! Spec: session-music-assistant, "Track and artwork", "Per-track position",
//! "Source quality", "Delivered quality", "Up next", "Music provider name" and "Album colours",
//! against recorded responses.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::path::PathBuf;
use std::time::Duration;

use jukelight_adapter_music_assistant::{Responses, to_content};
use jukelight_domain::{
    ArtworkRef, AudioFormat, AudioQuality, Brand, Change, Content, Decibels, Delivery, Palette,
    QueueView, Rgb, Timestamp,
};
use serde_json::{Value, json};

const MA_URL: &str = "http://192.168.1.50:8095";
const PLAYER_KEY: &str = "aabbccddeeff";

/// The four recorded responses of one situation, as JSON to adjust.
struct Recording {
    player: Value,
    queue: Value,
    items: Value,
    providers: Value,
    /// `players/all`; empty when the situation has none recorded.
    players: Value,
}

impl Recording {
    fn load(fixture: &str) -> Self {
        let read = |file: &str| -> Value {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(fixture)
                .join(file);
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
        };
        Self {
            player: read("player.json"),
            queue: read("queue.json"),
            items: read("items.json"),
            providers: read("providers.json"),
            players: if PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(fixture)
                .join("players.json")
                .exists()
            {
                read("players.json")
            } else {
                json!([])
            },
        }
    }

    fn responses(&self) -> Responses {
        Responses {
            player: serde_json::from_value(self.player.clone()).unwrap(),
            queue: serde_json::from_value(self.queue.clone()).unwrap(),
            upcoming: serde_json::from_value(self.items.clone()).unwrap(),
            providers: serde_json::from_value(self.providers.clone()).unwrap(),
            players: serde_json::from_value(self.players.clone()).unwrap(),
        }
    }

    /// When the queue measured its position, in milliseconds.
    fn measured_at(&self) -> Timestamp {
        seconds(self.queue["elapsed_time_last_updated"].as_f64().unwrap())
    }

    /// Content as mapped one second after the queue measured its position.
    fn content(&self) -> Content {
        let now = self.measured_at().plus(Duration::from_secs(1));
        to_content(&self.responses(), MA_URL, PLAYER_KEY, now).expect("content")
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn seconds(seconds: f64) -> Timestamp {
    Timestamp::from_millis((seconds * 1000.0) as u64)
}

mod track_and_artwork {
    use super::*;

    #[test]
    fn library_track() {
        let content = Recording::load("opensubsonic").content();
        assert_eq!(content.track.title, "Tantita Pena");
        assert_eq!(content.track.artists, ["Alejandro Fernández"]);
        assert_eq!(content.track.album.as_deref(), Some("Origenes"));
        assert_eq!(
            content.position.unwrap().duration,
            Some(Duration::from_secs(209))
        );
        assert_eq!(
            content.artwork,
            Some(ArtworkRef::new(
                "http://192.168.1.50:8095/imageproxy/2f126e4d2f7d993ba0cbd7adb7e9d4a8ea13bcfb6476a74b86c8559a12eb16e0?size=0&fmt=jpg"
            ))
        );
    }

    #[test]
    fn remote_image() {
        let content = Recording::load("qobuz").content();
        assert_eq!(content.track.title, "At the Purchaser's Option");
        assert_eq!(
            content.artwork,
            Some(ArtworkRef::new(
                "https://static.qobuz.com/images/covers/83/95/0075597939583_600.jpg"
            ))
        );
    }

    #[test]
    fn remote_image_that_is_not_http_uses_the_proxy() {
        let mut recording = Recording::load("qobuz");
        recording.queue["current_item"]["image"]["path"] = json!("file:///covers/cover.jpg");
        let artwork = recording.content().artwork.unwrap();
        assert!(
            artwork
                .source()
                .starts_with("http://192.168.1.50:8095/imageproxy/758336b4"),
            "{artwork:?}"
        );
    }

    #[test]
    fn current_media_when_the_queue_has_no_current_item() {
        let mut recording = Recording::load("qobuz");
        recording.queue["current_item"] = Value::Null;
        let content = recording.content();
        assert_eq!(content.track.title, "At the Purchaser's Option");
        assert_eq!(content.track.artists, ["Rhiannon Giddens"]);
        assert_eq!(content.track.album.as_deref(), Some("Freedom Highway"));
        assert_eq!(
            content.artwork,
            Some(ArtworkRef::new(
                "http://192.168.1.50:8095/imageproxy/758336b4501c623d60d59b95283d571e9f3afe9caae600cb0b70bc9dc87f0f5a?size=512&fmt=jpg"
            ))
        );
    }

    #[test]
    fn nothing_known() {
        let mut recording = Recording::load("qobuz");
        recording.queue["current_item"] = Value::Null;
        recording.player["current_media"] = Value::Null;
        let now = recording.measured_at();
        assert_eq!(
            to_content(&recording.responses(), MA_URL, PLAYER_KEY, now),
            None
        );
    }
}

mod per_track_position {
    use super::*;

    #[test]
    fn queue_position() {
        let mut recording = Recording::load("qobuz");
        recording.queue["elapsed_time"] = json!(171.8);
        recording.player["elapsed_time"] = json!(437.0);
        let measured_at = recording.measured_at();

        let position = recording.content().position.unwrap();
        assert_eq!(position.elapsed, Duration::from_millis(171_800));
        assert_eq!(position.observed_at, measured_at);
    }

    #[test]
    fn clock_skew() {
        let recording = Recording::load("qobuz");
        let now = recording.measured_at().plus(Duration::from_secs(1));
        let mut skewed = Recording::load("qobuz");
        skewed.queue["elapsed_time_last_updated"] =
            json!(skewed.queue["elapsed_time_last_updated"].as_f64().unwrap() + 3600.0);

        let content = to_content(&skewed.responses(), MA_URL, PLAYER_KEY, now).unwrap();
        assert_eq!(content.position.unwrap().observed_at, now);
    }
}

mod source_quality {
    use super::*;

    #[test]
    fn padded_cd_quality_source() {
        let quality = Recording::load("opensubsonic").content().quality.unwrap();
        assert_eq!(quality.label(), "FLAC 16/44.1");
        assert!(!quality.is_hi_res());
        assert_eq!(
            quality.detail.as_deref(),
            Some("Lossless · → 24/44.1 to player")
        );
    }

    #[test]
    fn normalised_hi_res_source() {
        let mut recording = Recording::load("qobuz");
        recording.queue["current_item"]["streamdetails"]["volume_normalization_mode"] =
            json!("dynamic");
        let quality = recording.content().quality.unwrap();
        assert_eq!(quality.label(), "FLAC 24/88.2");
        assert!(quality.is_hi_res());
        assert_eq!(quality.detail.as_deref(), Some("Normalised"));
    }

    #[test]
    fn unchanged_hi_res_source_has_no_detail() {
        let quality = Recording::load("qobuz").content().quality.unwrap();
        assert_eq!(quality.label(), "FLAC 24/88.2");
        assert_eq!(quality.detail, None);
    }

    #[test]
    fn lossy_source_reports_its_bitrate() {
        let mut recording = Recording::load("qobuz");
        recording.queue["current_item"]["streamdetails"]["audio_format"] =
            json!({"content_type": "mp3", "sample_rate": 44100, "bit_depth": 16, "bit_rate": 320});
        let quality = recording.content().quality.unwrap();
        assert_eq!(quality.label(), "320 kbps");
    }
}

// Spec: session-music-assistant, "Delivered quality".
mod delivered_quality {
    use super::*;

    fn flac(bit_depth: u32, sample_rate_hz: u32) -> AudioQuality {
        AudioQuality::format(AudioFormat {
            codec: "flac".into(),
            bit_depth,
            sample_rate_hz,
        })
    }

    fn details(recording: &mut Recording) -> &mut Value {
        &mut recording.queue["current_item"]["streamdetails"]
    }

    #[test]
    fn padded_cd_quality_source() {
        let delivery = Recording::load("opensubsonic").content().delivery;
        assert_eq!(
            delivery,
            Some(Delivery {
                quality: Some(flac(24, 44_100)),
                changes: vec![Change::BitDepth],
            })
        );
    }

    #[test]
    fn upsampled_and_normalised() {
        let mut recording = Recording::load("opensubsonic");
        let details = details(&mut recording);
        details["volume_normalization_mode"] = json!("dynamic");
        let processing = &mut details["audio_processing"];
        processing["queue_processing"]["normalization"]["applied_gain_db"] = json!(-6.0);
        let output = &mut processing["outputs"][0];
        output["output_format"]["sample_rate"] = json!(96000);
        output["dsp"]["state"] = json!("enabled");

        assert_eq!(
            recording.content().delivery,
            Some(Delivery {
                quality: Some(flac(24, 96_000)),
                changes: vec![
                    Change::Resampled,
                    Change::Normalised {
                        gain: Some(Decibels::from_db(-6.0))
                    },
                    Change::Dsp,
                ],
            })
        );
    }

    #[test]
    fn untouched_hi_res() {
        let delivery = Recording::load("qobuz").content().delivery;
        assert_eq!(
            delivery,
            Some(Delivery {
                quality: Some(flac(24, 88_200)),
                changes: vec![],
            })
        );
    }

    #[test]
    fn gain_falls_back_to_the_gain_correction() {
        let mut recording = Recording::load("qobuz");
        let details = details(&mut recording);
        details["volume_normalization_mode"] = json!("fixed_gain");
        details["audio_processing"]["queue_processing"]["normalization"] = Value::Null;
        details["volume_normalization_gain_correct"] = json!(-3.5);

        let delivery = recording.content().delivery.unwrap();
        assert_eq!(
            delivery.changes,
            vec![Change::Normalised {
                gain: Some(Decibels::from_db(-3.5))
            }]
        );
    }

    #[test]
    fn no_output_for_the_player() {
        let mut recording = Recording::load("qobuz");
        details(&mut recording)["audio_processing"]["outputs"] = json!([]);
        let delivery = recording.content().delivery.unwrap();
        assert_eq!(delivery.quality, None);
        assert_eq!(delivery.changes, vec![]);
    }

    #[test]
    fn no_stream_details() {
        let mut recording = Recording::load("qobuz");
        recording.queue["current_item"]["streamdetails"] = Value::Null;
        assert_eq!(recording.content().delivery, None);
    }
}

mod up_next {
    use super::*;

    #[test]
    fn following_tracks_with_thumbnails() {
        let content = Recording::load("qobuz").content();
        let QueueView::Available(entries) = content.queue else {
            panic!("queue not available")
        };
        let titles: Vec<_> = entries.iter().map(|entry| entry.title.as_str()).collect();
        assert_eq!(
            titles,
            [
                "Whole Lotta Love",
                "Keep On Shining",
                "Billie Jean",
                "Coming Home",
                "Johnny and Mary"
            ]
        );
        assert_eq!(entries[0].artists, ["Led Zeppelin"]);
        assert_eq!(
            entries[0].artwork,
            Some(ArtworkRef::new(
                "http://192.168.1.50:8095/imageproxy/e1e69c23e70d03b5c780cf328124b3e4de4559dbc8719321d9beb7225aefc051?size=256&fmt=jpg"
            ))
        );
    }

    #[test]
    fn at_most_five_entries() {
        let mut recording = Recording::load("qobuz");
        let mut items = recording.items.as_array().unwrap().clone();
        items.extend(items.clone());
        recording.items = Value::Array(items);
        let QueueView::Available(entries) = recording.content().queue else {
            panic!("queue not available")
        };
        assert_eq!(entries.len(), 5);
    }
}

mod music_provider_name {
    use super::*;

    #[test]
    fn qobuz_through_music_assistant() {
        let content = Recording::load("qobuz").content();
        assert_eq!(name(&content), Some("Qobuz"));
    }

    #[test]
    fn opensubsonic() {
        let content = Recording::load("opensubsonic").content();
        assert_eq!(name(&content), Some("Subsonic"));
    }

    #[test]
    fn built_in_provider_has_no_name() {
        let mut recording = Recording::load("qobuz");
        recording.queue["current_item"]["streamdetails"]["provider"] = json!("builtin");
        assert_eq!(recording.content().service, None);
    }

    #[test]
    fn unknown_provider_has_no_name() {
        let mut recording = Recording::load("qobuz");
        recording.queue["current_item"]["streamdetails"]["provider"] = json!("tidal--instance1");
        assert_eq!(recording.content().service, None);
    }
}

fn name(content: &Content) -> Option<&str> {
    content
        .service
        .as_ref()
        .map(|service| service.name.as_str())
}

fn brand(content: &Content) -> Option<&str> {
    content
        .service
        .as_ref()
        .and_then(|service| service.brand.as_ref())
        .map(Brand::as_str)
}

/// The recording's provider entry for `instance`, to rename or re-domain.
fn provider_entry<'a>(recording: &'a mut Recording, instance: &str) -> &'a mut Value {
    recording
        .providers
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|provider| provider["instance_id"] == instance)
        .unwrap()
}

// Spec: session-music-assistant, "Music provider brand".
mod music_provider_brand {
    use super::*;

    #[test]
    fn renamed_qobuz_provider() {
        let mut recording = Recording::load("qobuz");
        provider_entry(&mut recording, "qobuz--instance1")["name"] = json!("My Qobuz");
        let content = recording.content();
        assert_eq!(brand(&content), Some("qobuz"));
        assert_eq!(name(&content), Some("My Qobuz"));
    }

    #[test]
    fn navidrome() {
        let mut recording = Recording::load("opensubsonic");
        provider_entry(&mut recording, "opensubsonic--instance1")["name"] = json!("Navidrome");
        assert_eq!(brand(&recording.content()), Some("navidrome"));
    }

    #[test]
    fn another_subsonic_server() {
        let content = Recording::load("opensubsonic").content();
        assert_eq!(brand(&content), Some("subsonic"));
        assert_eq!(name(&content), Some("Subsonic"));
    }

    #[test]
    fn local_files() {
        let mut recording = Recording::load("opensubsonic");
        let entry = provider_entry(&mut recording, "opensubsonic--instance1");
        entry["domain"] = json!("filesystem_local");
        entry["name"] = json!("Example Folder");
        assert_eq!(brand(&recording.content()), Some("library"));
    }

    #[test]
    fn unknown_domain() {
        let mut recording = Recording::load("qobuz");
        let entry = provider_entry(&mut recording, "qobuz--instance1");
        entry["domain"] = json!("example_service");
        entry["name"] = json!("Example Service");
        let content = recording.content();
        assert_eq!(brand(&content), None);
        assert_eq!(name(&content), Some("Example Service"));
    }
}

mod album_colours {
    use super::*;

    #[test]
    fn palette_present() {
        let mut recording = Recording::load("qobuz");
        recording.player["current_media"]["palette"] = json!({
            "background_dark": [73, 51, 38],
            "on_dark": [209, 147, 108],
            "primary": [73, 51, 38],
        });
        assert_eq!(
            recording.content().palette,
            Some(Palette {
                background: Rgb(73, 51, 38),
                accent: Rgb(209, 147, 108),
            })
        );
    }

    #[test]
    fn recorded_palette() {
        assert_eq!(
            Recording::load("qobuz").content().palette,
            Some(Palette {
                background: Rgb(49, 49, 49),
                accent: Rgb(195, 195, 195),
            })
        );
    }

    #[test]
    fn incomplete_palette() {
        let mut recording = Recording::load("qobuz");
        recording.player["current_media"]["palette"] = json!({"background_dark": [73, 51, 38]});
        assert_eq!(recording.content().palette, None);
    }

    #[test]
    fn no_palette() {
        let mut recording = Recording::load("qobuz");
        recording.player["current_media"]["palette"] = Value::Null;
        assert_eq!(recording.content().palette, None);
    }
}

// Spec: session-music-assistant, "Claiming the session": which queue Music
// Assistant plays to the player from, by the recorded `players/get`.
mod music_assistant_queue {
    use jukelight_adapter_music_assistant::{api, music_assistant_queue};

    fn player(fixture: &str) -> api::Player {
        serde_json::from_value(super::fixture_json(fixture, "player.json")).unwrap()
    }

    #[test]
    fn music_assistant_plays_to_a_group() {
        assert_eq!(
            music_assistant_queue(&player("group-playing"), "aabbccddeeff").as_deref(),
            Some("aabbccddeeff")
        );
    }

    #[test]
    fn a_member_of_another_players_group() {
        let mut member = player("group-playing");
        member.active_source = Some("112233445566".into());
        member.synced_to = Some("112233445566".into());
        assert_eq!(
            music_assistant_queue(&member, "aabbccddeeff").as_deref(),
            Some("112233445566")
        );
    }

    #[test]
    fn airplay_from_a_phone() {
        assert_eq!(
            music_assistant_queue(&player("airplay-phone"), "aabbccddeeff"),
            None
        );
    }
}

fn fixture_json(fixture: &str, file: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture)
        .join(file);
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

// Spec: session-music-assistant, "The group's speakers".
mod the_groups_speakers {
    use super::*;
    use jukelight_domain::{AudioFormat, AudioQuality, Speaker, SpeakerKind};

    fn quality(codec: &str, bit_depth: u32, sample_rate_hz: u32) -> AudioQuality {
        AudioQuality::format(AudioFormat {
            codec: codec.into(),
            bit_depth,
            sample_rate_hz,
        })
    }

    #[test]
    fn recorded_group() {
        let speakers = Recording::load("group-playing").content().speakers;
        assert_eq!(
            speakers,
            vec![
                Speaker {
                    name: "Example Web Player".into(),
                    kind: SpeakerKind::Computer,
                    playing: true,
                    quality: Some(quality("flac", 24, 44_100)),
                },
                Speaker {
                    name: "Example Portable Speaker".into(),
                    kind: SpeakerKind::Speaker,
                    playing: true,
                    quality: Some(quality("s16le", 16, 44_100)),
                },
            ]
        );
    }

    #[test]
    fn a_speaker_dropped_out() {
        let mut recording = Recording::load("group-playing");
        let players = recording.players.as_array_mut().unwrap();
        let member = players
            .iter_mut()
            .find(|p| p["player_id"] == "up00000002")
            .unwrap();
        member["playback_state"] = json!("idle");
        let speakers = recording.content().speakers;
        assert!(
            !speakers
                .iter()
                .find(|s| s.name == "Example Portable Speaker")
                .unwrap()
                .playing
        );
    }

    #[test]
    fn no_format_for_a_speaker() {
        let mut recording = Recording::load("group-playing");
        recording.queue["current_item"]["streamdetails"]["audio_processing"]["outputs"] = json!([]);
        let speakers = recording.content().speakers;
        assert_eq!(speakers.len(), 2);
        assert!(speakers.iter().all(|s| s.quality.is_none()));
    }

    #[test]
    fn playing_alone() {
        assert_eq!(Recording::load("qobuz").content().speakers, vec![]);
    }

    #[test]
    fn a_member_sees_the_leader_and_the_others() {
        // This player is a member: the leader and the third player play along.
        let mut recording = Recording::load("group-playing");
        let players = recording.players.as_array_mut().unwrap();
        for p in players.iter_mut() {
            if p["player_id"] == "aabbccddeeff" {
                p["synced_to"] = json!("up00000002");
                p["group_members"] = json!([]);
            }
            if p["player_id"] == "up00000002" {
                p["synced_to"] = Value::Null;
                p["group_members"] = json!([
                    "up00000002",
                    "aabbccddeeff",
                    "ma_companion_00000000-0000-0000-0000-000000000006"
                ]);
            }
            if p["player_id"] == "ma_companion_00000000-0000-0000-0000-000000000006" {
                p["synced_to"] = json!("up00000002");
            }
        }
        recording.player["synced_to"] = json!("up00000002");
        recording.player["group_members"] = json!([]);
        let names: Vec<String> = recording
            .content()
            .speakers
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(names, ["Example Portable Speaker", "Example Web Player"]);
    }
}
