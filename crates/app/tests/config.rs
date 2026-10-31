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

mod music_assistant {
    use super::*;

    const TOKEN: &str = "example-token-0123456789";

    #[test]
    fn both_set() {
        let config = parse(&[
            ("BLUOS_HOST", "192.168.1.40"),
            ("MA_URL", "http://192.168.1.50:8095"),
            ("MA_TOKEN", TOKEN),
        ])
        .unwrap();
        assert!(
            !format!("{config:?}").contains(TOKEN),
            "token in Debug output"
        );
        let music_assistant = config.music_assistant.expect("Music Assistant configured");
        assert_eq!(music_assistant.url, "http://192.168.1.50:8095");
    }

    #[test]
    fn neither_set_means_no_provider() {
        let config = parse(&[("BLUOS_HOST", "192.168.1.40")]).unwrap();
        assert_eq!(config.music_assistant, None);
    }

    #[test]
    fn url_without_token() {
        let error = parse(&[
            ("BLUOS_HOST", "192.168.1.40"),
            ("MA_URL", "http://192.168.1.50:8095"),
        ])
        .unwrap_err();
        assert!(error.contains("MA_TOKEN"), "{error}");
    }

    #[test]
    fn token_without_url() {
        let error = parse(&[("BLUOS_HOST", "192.168.1.40"), ("MA_TOKEN", TOKEN)]).unwrap_err();
        assert!(error.contains("MA_URL"), "{error}");
        assert!(!error.contains(TOKEN), "token in the message: {error}");
    }

    #[test]
    fn url_must_be_http() {
        for url in ["ftp://192.168.1.50", "192.168.1.50:8095"] {
            let error = parse(&[
                ("BLUOS_HOST", "192.168.1.40"),
                ("MA_URL", url),
                ("MA_TOKEN", TOKEN),
            ])
            .unwrap_err();
            assert!(error.contains("MA_URL"), "{error}");
            assert!(!error.contains(TOKEN), "token in the message: {error}");
        }
    }
}

mod display_grace_period {
    use std::time::Duration;

    use super::*;

    #[test]
    fn ten_minutes_by_default() {
        let config = parse(&[("BLUOS_HOST", "192.168.1.40")]).unwrap();
        assert_eq!(config.display_grace, Duration::from_secs(600));
    }

    #[test]
    fn whole_minutes_from_0_to_1440() {
        for (value, minutes) in [("0", 0), ("3", 3), ("1440", 1440)] {
            let config = parse(&[
                ("BLUOS_HOST", "192.168.1.40"),
                ("DISPLAY_GRACE_MINUTES", value),
            ])
            .unwrap();
            assert_eq!(config.display_grace, Duration::from_secs(minutes * 60));
        }
    }

    #[test]
    fn invalid_values_name_the_variable() {
        for value in ["1441", "-1", "2.5", "ten"] {
            let error = parse(&[
                ("BLUOS_HOST", "192.168.1.40"),
                ("DISPLAY_GRACE_MINUTES", value),
            ])
            .unwrap_err();
            assert!(error.contains("DISPLAY_GRACE_MINUTES"), "{error}");
        }
    }
}

mod input_display_time {
    use std::time::Duration;

    use super::*;

    #[test]
    fn ninety_minutes_by_default() {
        let config = parse(&[("BLUOS_HOST", "192.168.1.40")]).unwrap();
        assert_eq!(config.input_display_limit, Duration::from_secs(90 * 60));
    }

    #[test]
    fn whole_minutes_from_0_to_1440() {
        for (value, minutes) in [("0", 0), ("80", 80), ("1440", 1440)] {
            let config = parse(&[
                ("BLUOS_HOST", "192.168.1.40"),
                ("INPUT_DISPLAY_MINUTES", value),
            ])
            .unwrap();
            assert_eq!(
                config.input_display_limit,
                Duration::from_secs(minutes * 60)
            );
        }
    }

    #[test]
    fn invalid_values_name_the_variable() {
        for value in ["1441", "-5", "1.5", "ninety"] {
            let error = parse(&[
                ("BLUOS_HOST", "192.168.1.40"),
                ("INPUT_DISPLAY_MINUTES", value),
            ])
            .unwrap_err();
            assert!(error.contains("INPUT_DISPLAY_MINUTES"), "{error}");
        }
    }
}
