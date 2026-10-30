//! Behaviour every `DeviceMonitor` must show, whatever the device.
//!
//! Each adapter runs [`device_monitor_contract`] with a harness that can make
//! its device unreachable and reachable again.

// A contract suite reports a broken rule by panicking, like an assertion.
#![allow(clippy::expect_used)]

use std::time::Duration;

use futures::StreamExt;
use futures::stream::BoxStream;
use jukelight_domain::DeviceSnapshot;

use crate::DeviceMonitor;

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
