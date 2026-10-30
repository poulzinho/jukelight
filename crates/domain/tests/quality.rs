//! Spec: playback-monitoring, "Audio quality" and "Hi-Res rule".

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use jukelight_domain::{AudioFormat, AudioQuality};

fn format(codec: &str, bit_depth: u32, sample_rate_hz: u32) -> AudioQuality {
    AudioQuality::format(AudioFormat {
        codec: codec.into(),
        bit_depth,
        sample_rate_hz,
    })
}

#[test]
fn lossless_format() {
    let quality = format("FLAC", 24, 96_000);

    assert_eq!(quality.label(), "FLAC 24/96");
    assert!(quality.is_lossless());
}

#[test]
fn bitrate_only() {
    let quality = AudioQuality::bitrate(160_000);

    assert_eq!(quality.label(), "160 kbps");
    assert!(!quality.is_lossless());
}

#[test]
fn sample_rates_with_decimals() {
    assert_eq!(format("FLAC", 16, 44_100).label(), "FLAC 16/44.1");
    assert_eq!(format("FLAC", 24, 88_200).label(), "FLAC 24/88.2");
    assert_eq!(format("FLAC", 24, 22_050).label(), "FLAC 24/22.05");
}

#[test]
fn detail_is_kept() {
    let quality = format("FLAC", 16, 44_100).with_detail("CD quality");
    assert_eq!(quality.detail.as_deref(), Some("CD quality"));
}

#[test]
fn cd_quality_is_not_hi_res() {
    assert!(!format("FLAC", 16, 44_100).is_hi_res());
}

#[test]
fn twenty_four_bit_at_44_1_is_hi_res() {
    assert!(format("FLAC", 24, 44_100).is_hi_res());
}

#[test]
fn high_sample_rate_is_hi_res() {
    assert!(format("FLAC", 24, 192_000).is_hi_res());
}

#[test]
fn lossy_codec_with_24_bits_is_not_hi_res() {
    assert!(!format("AAC", 24, 48_000).is_hi_res());
}

#[test]
fn bitrate_only_is_never_hi_res() {
    assert!(!AudioQuality::bitrate(320_000).is_hi_res());
}

#[test]
fn codec_names_are_case_insensitive() {
    assert!(format("flac", 24, 96_000).is_hi_res());
}
