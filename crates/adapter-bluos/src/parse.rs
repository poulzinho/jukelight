//! Parsing of BluOS responses. Pure functions from response text to typed
//! values, so tests need no network.
//!
//! BluOS responses are shallow: a root element whose children hold plain
//! text. Parsing is tolerant: unknown elements are kept, missing ones are
//! simply absent.

use std::collections::HashMap;

use quick_xml::events::{BytesRef, BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

use jukelight_domain::Brand;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid XML: {0}")]
    Xml(String),
    #[error("missing {0}")]
    Missing(&'static str),
}

/// `/Status`: its `etag` and the text of its direct children.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Status {
    pub etag: String,
    fields: HashMap<String, String>,
}

impl Status {
    /// The text of a direct child element, if present and not empty.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.fields
            .get(name)
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    }
}

/// The player's identity, from `/SyncStatus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub name: String,
    /// The MAC address in lowercase without separators.
    pub player_key: String,
    /// The player's maker, from `brand`, when it is a known brand.
    pub maker: Option<Brand>,
    /// `hasSubwoofer="true"`: BluOS leaves the attribute out while the
    /// subwoofer is switched off.
    pub has_subwoofer: bool,
    /// Changes whenever the player's settings change; `/Status` repeats it.
    pub sync_stat: Option<String>,
}

/// One `<song>` of `/Playlist`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistEntry {
    pub title: String,
    pub artist: Option<String>,
    pub image: Option<String>,
}

pub fn parse_status(xml: &str) -> Result<Status, ParseError> {
    let (root, children) = shallow(xml)?;
    let etag = root.get("etag").cloned().unwrap_or_default();
    let fields = children
        .into_iter()
        .map(|child| (child.name, child.text))
        .collect();
    Ok(Status { etag, fields })
}

pub fn parse_sync_status(xml: &str) -> Result<Identity, ParseError> {
    let (root, _) = shallow(xml)?;
    let name = root
        .get("name")
        .cloned()
        .ok_or(ParseError::Missing("name"))?;
    let mac = root.get("mac").ok_or(ParseError::Missing("mac"))?;
    let player_key = mac
        .chars()
        .filter(char::is_ascii_hexdigit)
        .collect::<String>()
        .to_lowercase();
    let maker = match root.get("brand").map(String::as_str) {
        Some("Bluesound") => Brand::new("bluesound"),
        _ => None,
    };
    Ok(Identity {
        name,
        player_key,
        maker,
        has_subwoofer: root.get("hasSubwoofer").map(String::as_str) == Some("true"),
        sync_stat: root.get("syncStat").cloned(),
    })
}

pub fn parse_playlist(xml: &str) -> Result<Vec<PlaylistEntry>, ParseError> {
    let mut reader = Reader::from_str(xml);
    let mut entries = Vec::new();
    let mut song: Option<HashMap<String, String>> = None;
    let mut field: Option<(String, String)> = None;

    loop {
        match reader.read_event().map_err(xml_error)? {
            Event::Start(start) => {
                let name = element_name(&start);
                if name == "song" {
                    song = Some(HashMap::new());
                } else if song.is_some() {
                    field = Some((name, String::new()));
                }
            }
            Event::Text(text) => {
                if let Some((_, value)) = field.as_mut() {
                    value.push_str(&text.decode().map_err(xml_error)?);
                }
            }
            Event::GeneralRef(reference) => {
                if let Some((_, value)) = field.as_mut() {
                    value.push_str(&resolve(&reference)?);
                }
            }
            Event::End(end) => {
                let name = String::from_utf8_lossy(end.name().as_ref()).into_owned();
                if name == "song" {
                    if let Some(fields) = song.take() {
                        entries.push(PlaylistEntry {
                            title: fields.get("title").cloned().unwrap_or_default(),
                            artist: fields.get("art").cloned().filter(|a| !a.is_empty()),
                            image: fields.get("image").cloned().filter(|i| !i.is_empty()),
                        });
                    }
                } else if let (Some((field_name, value)), Some(fields)) =
                    (field.take(), song.as_mut())
                {
                    fields.insert(field_name, value.trim().to_owned());
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(entries)
}

struct Child {
    name: String,
    text: String,
}

/// The root element's attributes and the text of its direct children.
/// Deeper elements (e.g. `<actions>`) are skipped.
fn shallow(xml: &str) -> Result<(HashMap<String, String>, Vec<Child>), ParseError> {
    let mut reader = Reader::from_str(xml);
    let mut root = None;
    let mut children = Vec::new();
    let mut depth = 0usize;
    let mut current: Option<Child> = None;

    loop {
        match reader.read_event().map_err(xml_error)? {
            Event::Start(start) => {
                depth += 1;
                match depth {
                    1 => root = Some(attributes(&start)?),
                    2 => {
                        current = Some(Child {
                            name: element_name(&start),
                            text: String::new(),
                        });
                    }
                    _ => {}
                }
            }
            Event::Empty(empty) => match depth {
                0 => root = Some(attributes(&empty)?),
                1 => children.push(Child {
                    name: element_name(&empty),
                    text: String::new(),
                }),
                _ => {}
            },
            Event::Text(text) if depth == 2 => {
                if let Some(child) = current.as_mut() {
                    child.text.push_str(&text.decode().map_err(xml_error)?);
                }
            }
            Event::GeneralRef(reference) if depth == 2 => {
                if let Some(child) = current.as_mut() {
                    child.text.push_str(&resolve(&reference)?);
                }
            }
            Event::End(_) => {
                if depth == 2
                    && let Some(mut child) = current.take()
                {
                    let trimmed = child.text.trim().to_owned();
                    child.text = trimmed;
                    children.push(child);
                }
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            _ => {}
        }
    }
    let root = root.ok_or(ParseError::Missing("root element"))?;
    Ok((root, children))
}

fn element_name(start: &BytesStart<'_>) -> String {
    String::from_utf8_lossy(start.name().as_ref()).into_owned()
}

fn attributes(start: &BytesStart<'_>) -> Result<HashMap<String, String>, ParseError> {
    start
        .attributes()
        .map(|attribute| {
            let attribute = attribute.map_err(xml_error)?;
            let key = String::from_utf8_lossy(attribute.key.as_ref()).into_owned();
            let value = attribute
                .normalized_value(XmlVersion::Implicit1_0)
                .map_err(xml_error)?
                .into_owned();
            Ok((key, value))
        })
        .collect()
}

/// `&amp;`, `&#39;` and friends.
fn resolve(reference: &BytesRef<'_>) -> Result<String, ParseError> {
    if let Some(character) = reference.resolve_char_ref().map_err(xml_error)? {
        return Ok(character.to_string());
    }
    let name = reference.decode().map_err(xml_error)?;
    let entity = format!("&{name};");
    Ok(quick_xml::escape::unescape(&entity).map_or(entity.clone(), std::borrow::Cow::into_owned))
}

fn xml_error(error: impl std::fmt::Display) -> ParseError {
    ParseError::Xml(error.to_string())
}
