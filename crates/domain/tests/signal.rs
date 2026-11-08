//! Spec: published-api, "Signal" (the quality class of a path).

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use jukelight_domain::{
    AudioFormat, AudioQuality, Change, Decibels, Delivery, QualityClass, signal_class,
};

fn format(codec: &str, bit_depth: u32, sample_rate_hz: u32) -> AudioQuality {
    AudioQuality::format(AudioFormat {
        codec: codec.into(),
        bit_depth,
        sample_rate_hz,
    })
}

fn delivered(quality: AudioQuality) -> Delivery {
    Delivery {
        quality: Some(quality),
        changes: vec![],
    }
}

#[test]
fn class_of_one_quality() {
    assert_eq!(format("FLAC", 24, 96_000).class(), QualityClass::HiRes);
    assert_eq!(format("FLAC", 16, 44_100).class(), QualityClass::Lossless);
    assert_eq!(format("MP3", 16, 44_100).class(), QualityClass::Lossy);
    assert_eq!(AudioQuality::bitrate(320_000).class(), QualityClass::Lossy);
}

#[test]
fn source_alone() {
    let source = format("FLAC", 24, 96_000);
    assert_eq!(signal_class(Some(&source), None), QualityClass::HiRes);
}

#[test]
fn upsampled_cd_quality_stays_lossless() {
    let source = format("FLAC", 16, 44_100);
    let delivery = delivered(format("FLAC", 24, 96_000));
    assert_eq!(
        signal_class(Some(&source), Some(&delivery)),
        QualityClass::Lossless
    );
}

#[test]
fn downsampled_hi_res_becomes_lossless() {
    let source = format("FLAC", 24, 192_000);
    let delivery = delivered(format("FLAC", 16, 48_000));
    assert_eq!(
        signal_class(Some(&source), Some(&delivery)),
        QualityClass::Lossless
    );
}

#[test]
fn unknown_delivery_keeps_the_source_class() {
    let source = format("FLAC", 24, 96_000);
    let delivery = Delivery {
        quality: None,
        changes: vec![Change::Dsp],
    };
    assert_eq!(
        signal_class(Some(&source), Some(&delivery)),
        QualityClass::HiRes
    );
}

#[test]
fn no_source_quality_is_unknown() {
    let delivery = delivered(format("FLAC", 24, 96_000));
    assert_eq!(signal_class(None, Some(&delivery)), QualityClass::Unknown);
    assert_eq!(signal_class(None, None), QualityClass::Unknown);
}

#[test]
fn gain_in_tenths_of_a_decibel() {
    assert_eq!(Decibels::from_db(-6.04), Decibels::from_db(-6.0));
    assert_eq!(Decibels::from_db(-6.06), Decibels::from_db(-6.1));
    assert!((Decibels::from_db(-6.06).as_db() + 6.1).abs() < 1e-9);
}
