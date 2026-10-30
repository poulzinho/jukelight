use jukelight_domain::{DeviceSnapshot, Origin};
use reqwest::Url;

/// Decides from configuration alone whether Music Assistant drives a device:
/// the device plays a stream from Music Assistant's host whose path names
/// the device's player key. The port is not compared, because Music
/// Assistant's stream server port is configurable.
#[derive(Debug, Clone)]
pub struct ClaimRule {
    host: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UrlError {
    #[error("the Music Assistant URL is not a valid URL")]
    Invalid,
    #[error("the Music Assistant URL must start with http:// or https://")]
    NotHttp,
}

impl ClaimRule {
    pub fn new(music_assistant_url: &str) -> Result<Self, UrlError> {
        let url = http_url(music_assistant_url)?;
        let host = url
            .host_str()
            .ok_or(UrlError::Invalid)?
            .to_ascii_lowercase();
        Ok(Self { host })
    }

    pub fn claims(&self, device: &DeviceSnapshot) -> bool {
        let Origin::Stream { url } = &device.origin else {
            return false;
        };
        let Ok(url) = Url::parse(url) else {
            return false;
        };
        !device.player_key.is_empty()
            && url
                .host_str()
                .is_some_and(|host| host.eq_ignore_ascii_case(&self.host))
            && url
                .path_segments()
                .is_some_and(|mut segments| segments.any(|segment| segment == device.player_key))
    }
}

/// Parses an `http(s)` URL.
pub(crate) fn http_url(text: &str) -> Result<Url, UrlError> {
    let url = Url::parse(text).map_err(|_| UrlError::Invalid)?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(UrlError::NotHttp);
    }
    Ok(url)
}
