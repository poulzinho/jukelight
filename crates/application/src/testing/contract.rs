//! Behaviour every `DeviceMonitor` and every `SessionProvider` must show.
//!
//! Each adapter runs [`device_monitor_contract`] or
//! [`session_provider_contract`] with a harness that controls the system
//! behind it.

// A contract suite reports a broken rule by panicking, like an assertion.
#![allow(clippy::expect_used)]

use std::time::Duration;

use futures::StreamExt;
use futures::stream::BoxStream;
use jukelight_domain::{Content, DeviceSnapshot};

use crate::{DeviceMonitor, SessionProvider};

/// How long the contract waits for an expected snapshot.
const WAIT: Duration = Duration::from_secs(5);

/// Controls the device behind a monitor under test.
pub trait DeviceHarness: Send + Sync {
    fn make_unreachable(&self);
    fn make_reachable(&self);
}

/// Runs the device monitor contract and panics with a description of the
/// first broken rule.
pub async fn device_monitor_contract(monitor: &dyn DeviceMonitor, harness: &dyn DeviceHarness) {
    let mut snapshots = monitor.snapshots();

    // 1. A reachable device is reported online, with its identity.
    let online = next_matching(&mut snapshots, |s| s.online)
        .await
        .expect("an online snapshot while the device is reachable");
    assert!(
        !online.name.is_empty(),
        "online snapshot without a device name"
    );
    assert!(
        !online.player_key.is_empty(),
        "online snapshot without a player key"
    );

    // 2. An unreachable device is reported offline, keeping the last content
    //    so the kiosk can keep showing it.
    harness.make_unreachable();
    let offline = next_matching(&mut snapshots, |s| !s.online)
        .await
        .expect("an offline snapshot after the device became unreachable");
    assert_eq!(
        offline.content.as_ref().map(|c| &c.track),
        online.content.as_ref().map(|c| &c.track)
    );

    // 3. The device is reported online again once it is back.
    harness.make_reachable();
    next_matching(&mut snapshots, |s| s.online)
        .await
        .expect("an online snapshot after the device came back");
}

async fn next_matching(
    snapshots: &mut BoxStream<'static, DeviceSnapshot>,
    matches: impl Fn(&DeviceSnapshot) -> bool,
) -> Option<DeviceSnapshot> {
    tokio::time::timeout(WAIT, async {
        while let Some(snapshot) = snapshots.next().await {
            if matches(&snapshot) {
                return Some(snapshot);
            }
        }
        None
    })
    .await
    .ok()
    .flatten()
}

/// Controls the system behind a session provider under test.
pub trait ProviderHarness: Send + Sync {
    /// A device report the provider must claim.
    fn claimed_device(&self) -> DeviceSnapshot;
    /// A device report the provider must not claim.
    fn unclaimed_device(&self) -> DeviceSnapshot;
    fn make_unreachable(&self);
    fn make_reachable(&self);
    /// Makes the provider's source report different content.
    fn change_content(&self);
}

/// Runs the session provider contract and panics with a description of the
/// first broken rule.
pub async fn session_provider_contract(
    provider: &dyn SessionProvider,
    harness: &dyn ProviderHarness,
) {
    // 1. Claims are decided offline, from configuration and the device alone.
    harness.make_unreachable();
    assert!(
        provider.claims(&harness.claimed_device()),
        "claim must not depend on reaching the provider"
    );
    assert!(
        !provider.claims(&harness.unclaimed_device()),
        "must not claim a device it does not drive"
    );

    // 2. While reachable and followed, the provider has content.
    harness.make_reachable();
    provider.follow(&harness.claimed_device());
    let first = wait_for(provider, Option::is_some)
        .await
        .expect("content while the provider is reachable");

    // 3. A content change publishes an update tick and new content.
    let mut updates = provider.updates();
    harness.change_content();
    tokio::time::timeout(WAIT, updates.next())
        .await
        .expect("an update tick after the content changed")
        .expect("the update stream must not end");
    wait_for(provider, |content| {
        content.is_some() && *content != Some(first.clone())
    })
    .await
    .expect("new content after the change");

    // 4. Unreachable: no content. 5. Back: content again.
    harness.make_unreachable();
    wait_for(provider, Option::is_none)
        .await
        .expect("no content while the provider is unreachable");
    harness.make_reachable();
    wait_for(provider, Option::is_some)
        .await
        .expect("content again after the provider came back");
}

/// Polls `provider.content()` until it matches, for at most `WAIT`.
async fn wait_for(
    provider: &dyn SessionProvider,
    matches: impl Fn(&Option<Content>) -> bool,
) -> Option<Content> {
    let deadline = tokio::time::Instant::now() + WAIT;
    loop {
        let content = provider.content();
        if matches(&content) {
            return Some(content.unwrap_or_else(placeholder));
        }
        if tokio::time::Instant::now() >= deadline {
            return None;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

/// Stands in for "matched no content" so `wait_for` can return `Some`.
fn placeholder() -> Content {
    Content {
        track: jukelight_domain::Track {
            title: String::new(),
            artists: Vec::new(),
            album: None,
        },
        artwork: None,
        position: None,
        quality: None,
        delivery: None,
        queue: jukelight_domain::QueueView::NotApplicable,
        service: None,
        palette: None,
        speakers: Vec::new(),
    }
}
