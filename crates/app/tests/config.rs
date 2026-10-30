//! Configuration from environment variables (design decision 10).

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::collections::HashMap;

use jukelight::config::{Config, DeviceConfig};

fn parse(pairs: &[(&str, &str)]) -> Result<Config, String> {
    let env: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    Config::from_lookup(|key| env.get(key).cloned()).map_err(|e| e.to_string())
}

#[test]
fn bluos_with_defaults() {
    let config = parse(&[("BLUOS_HOST", "192.168.1.40")]).unwrap();
    assert_eq!(
        config.device,
        DeviceConfig::Bluos {
            host: "192.168.1.40".into(),
            port: 11000
        }
    );
    assert_eq!(config.http_port, 8080);
    assert_eq!(config.log_level, "info");
}

#[test]
fn bluos_host_is_required() {
    let error = parse(&[]).unwrap_err();
    assert!(error.contains("BLUOS_HOST"), "{error}");
}

#[test]
fn explicit_values() {
    let config = parse(&[
        ("BLUOS_HOST", "player.local"),
        ("BLUOS_PORT", "11001"),
        ("HTTP_PORT", "9090"),
        ("LOG_LEVEL", "debug"),
    ])
    .unwrap();
    assert_eq!(
        config.device,
        DeviceConfig::Bluos {
            host: "player.local".into(),
            port: 11001
        }
    );
    assert_eq!(config.http_port, 9090);
    assert_eq!(config.log_level, "debug");
}

#[test]
fn fake_device() {
    let config = parse(&[("JUKELIGHT_DEVICE", "fake:paused")]).unwrap();
    assert_eq!(
        config.device,
        DeviceConfig::Fake {
            scenario: "paused".into()
        }
    );
}

#[test]
fn fake_device_ignores_bluos_host() {
    let config = parse(&[
        ("JUKELIGHT_DEVICE", "fake:paused"),
        ("BLUOS_HOST", "192.168.1.40"),
    ])
    .unwrap();
    assert!(matches!(config.device, DeviceConfig::Fake { .. }));
}

#[test]
fn unknown_fake_scenario() {
    let error = parse(&[("JUKELIGHT_DEVICE", "fake:nope")]).unwrap_err();
    assert!(
        error.contains("playing-with-queue"),
        "should list scenarios: {error}"
    );
}

#[test]
fn unknown_device_kind() {
    let error = parse(&[("JUKELIGHT_DEVICE", "sonos")]).unwrap_err();
    assert!(error.contains("JUKELIGHT_DEVICE"), "{error}");
}

#[test]
fn invalid_ports() {
    for (key, value) in [
        ("BLUOS_PORT", "abc"),
        ("HTTP_PORT", "0"),
        ("HTTP_PORT", "70000"),
    ] {
        let error = parse(&[("BLUOS_HOST", "192.168.1.40"), (key, value)]).unwrap_err();
        assert!(error.contains(key), "{key}={value}: {error}");
    }
}

#[test]
fn invalid_log_level() {
    let error = parse(&[("BLUOS_HOST", "192.168.1.40"), ("LOG_LEVEL", "loud")]).unwrap_err();
    assert!(error.contains("LOG_LEVEL"), "{error}");
}

#[test]
fn empty_values_count_as_missing() {
    let config = parse(&[("BLUOS_HOST", "192.168.1.40"), ("HTTP_PORT", "")]).unwrap();
    assert_eq!(config.http_port, 8080);
}
