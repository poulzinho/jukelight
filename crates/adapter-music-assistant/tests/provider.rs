//! Spec: session-music-assistant, "Claiming the session" (group sessions),
//! "The group's queue", "Following the player", "Up next" and
//! "Unreachable Music Assistant", against a local test server that replays
//! recorded responses; plus the session provider contract.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

mod support;

use std::time::Duration;

use futures::StreamExt;
use jukelight_adapter_music_assistant::Timing;
use jukelight_application::SessionProvider;
use jukelight_application::testing::contract::{ProviderHarness, session_provider_contract};
use jukelight_domain::DeviceSnapshot;
use serde_json::json;
use support::*;

mod following_the_player {
    use super::*;

    #[tokio::test]
    async fn claimed() {
        let server = TestServer::start("qobuz").await;
        let provider = server.provider(TOKEN, Timing::default());
        provider.follow(&claimed_device());

        tokio::time::sleep(Duration::from_millis(2500)).await;
        assert!(server.requests("players/get").len() >= 2);
        assert!(server.requests("player_queues/get").len() >= 2);
        assert_eq!(
            server.requests("players/get")[0],
            json!({"player_id": "aabbccddeeff"})
        );
    }

    #[tokio::test]
    async fn not_claimed() {
        let server = TestServer::start("qobuz").await;
        let provider = server.provider(TOKEN, FAST);
        provider.follow(&service_device());

        tokio::time::sleep(Duration::from_secs(1)).await;
        assert_eq!(server.request_count(), 0);
        assert_eq!(provider.content(), None);
    }

    #[tokio::test]
    async fn stops_when_no_longer_claimed() {
        let server = TestServer::start("qobuz").await;
        let provider = server.provider(TOKEN, FAST);
        provider.follow(&claimed_device());
        wait_for(&provider, Option::is_some).await;

        provider.follow(&service_device());
        tokio::time::sleep(Duration::from_millis(300)).await;
        let count = server.request_count();
        tokio::time::sleep(Duration::from_millis(500)).await;
        assert_eq!(server.request_count(), count);
    }

    #[tokio::test]
    async fn track_change() {
        let server = TestServer::start("opensubsonic").await;
        let provider = server.provider(TOKEN, FAST);
        provider.follow(&claimed_device());
        wait_for(&provider, |c| title(c.as_ref()) == Some("Tantita Pena")).await;

        let mut updates = provider.updates();
        server.set_fixture("qobuz");
        tokio::time::timeout(WAIT, updates.next())
            .await
            .expect("an update after the track change")
            .expect("updates continue");
        wait_for(&provider, |c| {
            title(c.as_ref()) == Some("At the Purchaser's Option")
        })
        .await;
    }
}

mod up_next {
    use super::*;

    #[tokio::test]
    async fn following_tracks() {
        let server = TestServer::start("qobuz").await;
        server.patch_queue(json!({"items": 11, "current_index": 2}));
        let provider = server.provider(TOKEN, FAST);
        provider.follow(&claimed_device());
        wait_for(&provider, Option::is_some).await;

        // Items 4 to 8: the five after index 2.
        assert_eq!(
            server.requests("player_queues/items")[0],
            json!({"queue_id": "aabbccddeeff", "limit": 5, "offset": 3})
        );
    }

    #[tokio::test]
    async fn unchanged_queue_fetched_once() {
        let server = TestServer::start("qobuz").await;
        let provider = server.provider(TOKEN, FAST);
        provider.follow(&claimed_device());
        wait_for(&provider, Option::is_some).await;
        tokio::time::sleep(Duration::from_millis(300)).await;

        assert!(server.requests("player_queues/get").len() >= 3);
        assert_eq!(server.requests("player_queues/items").len(), 1);
        assert_eq!(server.requests("providers").len(), 1);
    }

    #[tokio::test]
    async fn changed_queue_fetched_again() {
        let server = TestServer::start("qobuz").await;
        let provider = server.provider(TOKEN, FAST);
        provider.follow(&claimed_device());
        wait_for(&provider, Option::is_some).await;

        server.patch_queue(json!({"current_index": 1}));
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(server.requests("player_queues/items").len(), 2);
    }
}

mod unreachable_music_assistant {
    use super::*;

    #[tokio::test]
    async fn server_down() {
        let server = TestServer::start("qobuz").await;
        server.set_reachable(false);
        let provider = server.provider(TOKEN, FAST);
        provider.follow(&claimed_device());

        tokio::time::sleep(Duration::from_millis(500)).await;
        assert_eq!(provider.content(), None);
        assert!(provider.claims(&claimed_device()));
    }

    #[tokio::test]
    async fn wrong_token() {
        let server = TestServer::start("qobuz").await;
        let provider = server.provider("wrong-token-0123456789", FAST);
        provider.follow(&claimed_device());

        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(provider.content(), None);
    }

    #[tokio::test]
    async fn back() {
        let server = TestServer::start("qobuz").await;
        let provider = server.provider(TOKEN, FAST);
        provider.follow(&claimed_device());
        wait_for(&provider, Option::is_some).await;

        server.set_reachable(false);
        wait_for(&provider, Option::is_none).await;
        server.set_reachable(true);
        wait_for(&provider, Option::is_some).await;
    }
}

mod contract {
    use super::*;

    struct Harness(TestServer);

    impl ProviderHarness for Harness {
        fn claimed_device(&self) -> DeviceSnapshot {
            claimed_device()
        }
        fn unclaimed_device(&self) -> DeviceSnapshot {
            service_device()
        }
        fn make_unreachable(&self) {
            self.0.set_reachable(false);
        }
        fn make_reachable(&self) {
            self.0.set_reachable(true);
        }
        fn change_content(&self) {
            self.0.set_fixture("opensubsonic");
        }
    }

    #[tokio::test]
    async fn music_assistant_provider_meets_the_contract() {
        let server = TestServer::start("qobuz").await;
        let provider = server.provider(TOKEN, FAST);
        session_provider_contract(provider.as_ref(), &Harness(server)).await;
    }
}

// Spec: session-music-assistant, "Claiming the session" (Music Assistant's
// group sessions), "Following the player" ("AirPlay session") and "The
// group's queue".
mod group_sessions {
    use super::*;

    #[tokio::test]
    async fn music_assistant_plays_to_a_group() {
        let server = TestServer::start("group-playing").await;
        let provider = server.provider(TOKEN, FAST);
        let device = airplay_device();
        provider.follow(&device);
        wait_for_claim(&provider, &device, true).await;
    }

    #[tokio::test]
    async fn the_player_leads_the_group() {
        let server = TestServer::start("group-playing").await;
        let provider = server.provider(TOKEN, FAST);
        provider.follow(&airplay_device());
        let content = wait_for(&provider, Option::is_some).await;
        assert_eq!(
            title(content.as_ref()),
            Some(
                fixture("group-playing", "queue.json")["current_item"]["media_item"]["name"]
                    .as_str()
                    .unwrap()
            )
        );
        assert_eq!(
            server.requests("player_queues/get")[0],
            json!({"queue_id": "aabbccddeeff"})
        );
    }

    #[tokio::test]
    async fn a_member_of_another_players_group() {
        let server = TestServer::start("group-playing").await;
        server.patch_player(json!({"active_source": "112233445566", "synced_to": "112233445566"}));
        let provider = server.provider(TOKEN, FAST);
        let device = airplay_device();
        provider.follow(&device);
        wait_for_claim(&provider, &device, true).await;
        wait_for(&provider, Option::is_some).await;
        assert_eq!(
            server.requests("player_queues/get")[0],
            json!({"queue_id": "112233445566"})
        );
    }

    #[tokio::test]
    async fn airplay_from_a_phone() {
        let server = TestServer::start("airplay-phone").await;
        let provider = server.provider(TOKEN, FAST);
        let device = airplay_device();
        provider.follow(&device);

        tokio::time::sleep(Duration::from_millis(500)).await;
        assert!(!provider.claims(&device));
        assert_eq!(provider.content(), None);
        // "AirPlay session": only the player is asked about.
        assert!(server.requests("players/get").len() >= 2);
        assert_eq!(server.requests("player_queues/get").len(), 0);
    }

    #[tokio::test]
    async fn no_answer_yet() {
        let server = TestServer::start("group-playing").await;
        let provider = server.provider(TOKEN, FAST);
        let device = airplay_device();
        provider.follow(&device);
        wait_for_claim(&provider, &device, true).await;

        // Music Assistant stops answering: the claim lapses.
        server.set_reachable(false);
        wait_for_claim(&provider, &device, false).await;
    }

    #[tokio::test]
    async fn a_new_answer_rebuilds_the_snapshot() {
        let server = TestServer::start("airplay-phone").await;
        let provider = server.provider(TOKEN, FAST);
        let device = airplay_device();
        provider.follow(&device);
        tokio::time::sleep(Duration::from_millis(200)).await;

        let mut updates = provider.updates();
        server.set_fixture("group-playing");
        tokio::time::timeout(WAIT, updates.next())
            .await
            .expect("an update once Music Assistant drives the player")
            .expect("updates continue");
        wait_for_claim(&provider, &device, true).await;
    }
}
