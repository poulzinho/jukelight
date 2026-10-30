//! Configuration from environment variables. Invalid configuration stops
//! the process at start, with a message naming the variable.

use jukelight_adapter_fake::SCENARIOS;
use jukelight_adapter_music_assistant::{ClaimRule, MusicAssistantConfig, Token};

const LOG_LEVELS: [&str; 5] = ["trace", "debug", "info", "warn", "error"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceConfig {
    Bluos { host: String, port: u16 },
    Fake { scenario: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub device: DeviceConfig,
    pub http_port: u16,
    pub log_level: String,
    /// `None`: Music Assistant streams show as plain streams.
    pub music_assistant: Option<MusicAssistantConfig>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0} is required (the address of your BluOS player, e.g. 192.168.1.40)")]
    Missing(&'static str),
    #[error("{set} is set but {missing} is not; set both or neither")]
    Incomplete {
        set: &'static str,
        missing: &'static str,
    },
    #[error("{key}={value:?} is invalid: {reason}")]
    Invalid {
        key: &'static str,
        value: String,
        reason: String,
    },
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// Reads the configuration through `lookup`; empty values count as unset.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let get = |key: &str| lookup(key).filter(|value| !value.trim().is_empty());

        let device = match get("JUKELIGHT_DEVICE") {
            Some(value) => match value.strip_prefix("fake:") {
                Some(scenario) if SCENARIOS.contains(&scenario) => DeviceConfig::Fake {
                    scenario: scenario.to_owned(),
                },
                Some(_) => {
                    return Err(invalid(
                        "JUKELIGHT_DEVICE",
                        &value,
                        format!("unknown scenario; choose fake:<{}>", SCENARIOS.join("|")),
                    ));
                }
                None => {
                    return Err(invalid(
                        "JUKELIGHT_DEVICE",
                        &value,
                        "use fake:<scenario>".into(),
                    ));
                }
            },
            None => DeviceConfig::Bluos {
                host: get("BLUOS_HOST").ok_or(ConfigError::Missing("BLUOS_HOST"))?,
                port: port("BLUOS_PORT", get("BLUOS_PORT"), 11000)?,
            },
        };

        let log_level = get("LOG_LEVEL")
            .unwrap_or_else(|| "info".into())
            .to_lowercase();
        if !LOG_LEVELS.contains(&log_level.as_str()) {
            return Err(invalid(
                "LOG_LEVEL",
                &log_level,
                format!("choose one of {}", LOG_LEVELS.join(", ")),
            ));
        }

        Ok(Self {
            device,
            http_port: port("HTTP_PORT", get("HTTP_PORT"), 8080)?,
            log_level,
            music_assistant: music_assistant(get("MA_URL"), get("MA_TOKEN"))?,
        })
    }
}

/// Both or neither. The token never appears in an error message.
fn music_assistant(
    url: Option<String>,
    token: Option<String>,
) -> Result<Option<MusicAssistantConfig>, ConfigError> {
    match (url, token) {
        (None, None) => Ok(None),
        (Some(_), None) => Err(ConfigError::Incomplete {
            set: "MA_URL",
            missing: "MA_TOKEN",
        }),
        (None, Some(_)) => Err(ConfigError::Incomplete {
            set: "MA_TOKEN",
            missing: "MA_URL",
        }),
        (Some(url), Some(token)) => {
            let url = url.trim().to_owned();
            ClaimRule::new(&url).map_err(|error| invalid("MA_URL", &url, error.to_string()))?;
            Ok(Some(MusicAssistantConfig {
                url,
                token: Token::new(token.trim()),
            }))
        }
    }
}

fn port(key: &'static str, value: Option<String>, default: u16) -> Result<u16, ConfigError> {
    let Some(value) = value else {
        return Ok(default);
    };
    match value.trim().parse::<u16>() {
        Ok(port) if port > 0 => Ok(port),
        _ => Err(invalid(
            key,
            &value,
            "expected a port number from 1 to 65535".into(),
        )),
    }
}

fn invalid(key: &'static str, value: &str, reason: String) -> ConfigError {
    ConfigError::Invalid {
        key,
        value: value.to_owned(),
        reason,
    }
}
