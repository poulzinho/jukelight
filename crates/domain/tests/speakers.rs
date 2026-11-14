//! Spec: session-music-assistant, "The group's speakers"; session-arbitration,
//! "Speakers and outputs on the snapshot": the domain's speakers and outputs.

use jukelight_domain::{
    AudioFormat, AudioQuality, Output, OutputKind, QualityClass, Speaker, SpeakerKind,
};

fn speaker(quality: Option<AudioQuality>) -> Speaker {
    Speaker {
        name: "Kitchen".into(),
        kind: SpeakerKind::Speaker,
        playing: true,
        quality,
    }
}

#[test]
fn a_speakers_class_follows_what_it_receives() {
    let pcm = AudioQuality::format(AudioFormat {
        codec: "s16le".into(),
        bit_depth: 16,
        sample_rate_hz: 44_100,
    });
    assert_eq!(speaker(Some(pcm)).class(), QualityClass::Lossless);
    assert_eq!(
        speaker(Some(AudioQuality::bitrate(256_000))).class(),
        QualityClass::Lossy
    );
    assert_eq!(speaker(None).class(), QualityClass::Unknown);
}

#[test]
fn pcm_is_lossless() {
    for codec in ["pcm", "s16le", "s24le", "s32le", "f32le"] {
        let quality = AudioQuality::format(AudioFormat {
            codec: codec.into(),
            bit_depth: 16,
            sample_rate_hz: 44_100,
        });
        assert!(quality.is_lossless(), "{codec}");
    }
}

#[test]
fn outputs_name_what_the_player_drives() {
    let speaker = Output {
        name: Some("Example Speaker".into()),
        kind: OutputKind::Speaker,
    };
    let subwoofer = Output {
        name: None,
        kind: OutputKind::Subwoofer,
    };
    assert_eq!(subwoofer.display_name(), "Subwoofer");
    assert_eq!(speaker.display_name(), "Example Speaker");
}

#[test]
fn padded_cd_quality_is_not_hi_res() {
    let flac = |bit_depth, sample_rate_hz| {
        AudioQuality::format(AudioFormat {
            codec: "flac".into(),
            bit_depth,
            sample_rate_hz,
        })
    };
    let office = speaker(Some(flac(24, 44_100)));
    assert_eq!(office.class(), QualityClass::HiRes);
    assert_eq!(
        office.class_from(Some(&flac(16, 44_100))),
        QualityClass::Lossless
    );
    assert_eq!(
        office.class_from(Some(&flac(24, 96_000))),
        QualityClass::HiRes
    );
}
