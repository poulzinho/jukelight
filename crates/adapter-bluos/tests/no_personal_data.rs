//! Spec: device-bluos, "Recorded responses as tests": committed fixtures
//! contain example values only.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

const EXAMPLE_MAC: &str = "aabbccddeeff";

/// Problems found in one text: real-looking IP or MAC addresses.
fn personal_values(text: &str) -> Vec<String> {
    let ipv4 = Regex::new(r"\b(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})\b").unwrap();
    let mac = Regex::new(r"(?i)\b(?:[0-9a-f]{2}[:-]){5}[0-9a-f]{2}\b").unwrap();
    // A MAC without separators: exactly 12 hex characters on their own.
    let bare_mac = Regex::new(r"(?i)(?:^|[^0-9a-z])([0-9a-f]{12})(?:$|[^0-9a-z])").unwrap();

    let mut found = Vec::new();
    for ip in ipv4.find_iter(text) {
        if !ip.as_str().starts_with("192.168.1.") && ip.as_str() != "127.0.0.1" {
            found.push(format!("IP address {}", ip.as_str()));
        }
    }
    for address in mac.find_iter(text) {
        let normalized: String = address
            .as_str()
            .chars()
            .filter(char::is_ascii_hexdigit)
            .collect();
        if !normalized.eq_ignore_ascii_case(EXAMPLE_MAC) {
            found.push(format!("MAC address {}", address.as_str()));
        }
    }
    for captures in bare_mac.captures_iter(text) {
        let candidate = &captures[1];
        if !candidate.eq_ignore_ascii_case(EXAMPLE_MAC) {
            found.push(format!("MAC-like value {candidate}"));
        }
    }
    found
}

/// Every committed fixture file (the git-ignored `raw` folder excluded).
fn fixture_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name != "raw") {
                files.extend(fixture_files(&path));
            }
        } else {
            files.push(path);
        }
    }
    files
}

#[test]
fn no_personal_data_in_fixtures() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let files = fixture_files(&dir);
    assert!(!files.is_empty(), "no fixtures found in {}", dir.display());

    let problems: Vec<String> = files
        .iter()
        .flat_map(|file| {
            let text = fs::read_to_string(file).unwrap();
            personal_values(&text)
                .into_iter()
                .map(move |problem| format!("{}: {problem}", file.display()))
        })
        .collect();
    assert!(
        problems.is_empty(),
        "personal values in fixtures:\n{}",
        problems.join("\n")
    );
}

#[test]
fn example_values_pass() {
    let text = r#"<SyncStatus id="192.168.1.40:11000" mac="AA:BB:CC:DD:EE:FF"/>
        <streamUrl>http://192.168.1.50:8097/flow/session1/aabbccddeeff/x.flac</streamUrl>"#;
    assert_eq!(personal_values(text), Vec::<String>::new());
}

#[test]
fn real_looking_values_are_found() {
    let text = r#"<SyncStatus id="10.0.0.7:11000" mac="12:34:56:78:9A:BC"/>
        <streamUrl>http://10.0.0.8:8097/flow/s/123456789abc/x.flac</streamUrl>"#;
    let problems = personal_values(text);
    assert_eq!(problems.len(), 4, "{problems:?}");
}

#[test]
fn etags_are_not_mistaken_for_macs() {
    let text = r#"<status etag="016a10146240aa84c67b0f3c8a7fb215">"#;
    assert_eq!(personal_values(text), Vec::<String>::new());
}
