//! Spec: session-music-assistant, "Token handling". A test binary of its
//! own: the log capture uses a thread-local subscriber, which tests running
//! on other threads would race with.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

mod support;

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use jukelight_adapter_music_assistant::{MusicAssistantConfig, Token};
use jukelight_application::SessionProvider;
use support::*;

/// Collects log output in memory.
#[derive(Clone, Default)]
struct Logs(Arc<Mutex<Vec<u8>>>);

impl Write for Logs {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Logs {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

// Single-threaded so the provider's task logs to this test's subscriber.
#[tokio::test(flavor = "current_thread")]
async fn failed_request_is_logged_without_the_token() {
    let logs = Logs::default();
    let writer = logs.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let server = TestServer::start("qobuz").await;
    let wrong = "wrong-token-0123456789";
    let provider = server.provider(wrong, FAST);
    provider.follow(&claimed_device());
    tokio::time::sleep(Duration::from_millis(300)).await;
    server.set_reachable(false);
    tokio::time::sleep(Duration::from_millis(500)).await;

    let text = logs.text();
    assert!(text.contains("Music Assistant"), "nothing logged: {text}");
    assert!(!text.contains(wrong), "token in the logs: {text}");
    assert!(!text.contains(&server.url), "URL in the logs: {text}");
}

#[test]
fn token_is_not_printed() {
    let token = Token::new(TOKEN);
    assert_eq!(format!("{token:?}"), "[redacted]");
    let config = MusicAssistantConfig {
        url: "http://192.168.1.50:8095".into(),
        token,
    };
    assert!(!format!("{config:?}").contains(TOKEN));
}
