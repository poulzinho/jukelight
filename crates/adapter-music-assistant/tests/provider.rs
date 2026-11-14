//! Spec: session-music-assistant, "Claiming the session" (group sessions),
//! "The group's queue", "Following the player", "Up next" and
//! "Unreachable Music Assistant" and "A paused group", against a local test server that replays
//! recorded responses; plus the session provider contract.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

mod support;

use std::time::Duration;

use futures::StreamExt;
use jukelight_adapter_music_assistant::Timing;
use jukelight_application::SessionProvider;
use jukelight_application::testing::contract::{ProviderHarness, session_provider_contract};
use jukelight_domain::{DeviceSnapshot, PlaybackState};
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
mod paused_groups {
    use super::*;

    /// The player after Music Assistant paused the group: stopped, back on
    /// its own previous queue.
    fn stopped_on_its_own_queue() -> DeviceSnapshot {
        DeviceSnapshot {
            state: PlaybackState::Stopped,
            ..service_device()
        }
    }

    fn paused_title() -> String {
        fixture("group-paused", "queue.json")["current_item"]["media_item"]["name"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    /// A group session, then Music Assistant pauses it.
    async fn held(
        server: &TestServer,
    ) -> std::sync::Arc<jukelight_adapter_music_assistant::MusicAssistantProvider> {
        let provider = server.provider(TOKEN, FAST);
        provider.follow(&airplay_device());
        wait_for(&provider, |content| {
            content.as_ref().is_some_and(|content| !content.held)
        })
        .await;
        server.set_fixture("group-paused");
        let stopped = stopped_on_its_own_queue();
        provider.follow(&stopped);
        wait_for_claim(&provider, &stopped, true).await;
        provider
    }

    #[tokio::test]
    async fn group_paused() {
        let server = TestServer::start("group-playing").await;
        let provider = held(&server).await;
        // Held at once with what played, then at the paused queue's position.
        let content = wait_for(&provider, |content| {
            content.as_ref().is_some_and(|content| {
                content.held
                    && content
                        .position
                        .is_some_and(|position| position.elapsed.as_secs() == 121)
            })
        })
        .await
        .unwrap();
        assert_eq!(content.track.title, paused_title());
        assert!(!content.speakers.is_empty(), "the group's speakers stay");
    }

    #[tokio::test]
    async fn group_resumed() {
        let server = TestServer::start("group-playing").await;
        let provider = held(&server).await;
        server.set_fixture("group-playing");
        let device = airplay_device();
        provider.follow(&device);
        wait_for_claim(&provider, &device, true).await;
        wait_for(&provider, |content| {
            content.as_ref().is_some_and(|content| !content.held)
        })
        .await;
    }

    #[tokio::test]
    async fn something_else_plays() {
        let server = TestServer::start("group-playing").await;
        let provider = held(&server).await;
        let device = service_device();
        provider.follow(&device);
        assert!(!provider.claims(&device));
        // Back to the stopped player: the hold is over.
        let stopped = stopped_on_its_own_queue();
        provider.follow(&stopped);
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(!provider.claims(&stopped));
    }

    #[tokio::test]
    async fn queue_cleared() {
        let server = TestServer::start("group-playing").await;
        let provider = held(&server).await;
        server.patch_queue(json!({"current_item": null, "current_index": null}));
        let stopped = stopped_on_its_own_queue();
        wait_for_claim(&provider, &stopped, false).await;
        assert_eq!(provider.content(), None);
        // The hold is over: no more requests.
        tokio::time::sleep(Duration::from_millis(100)).await;
        let count = server.request_count();
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(server.request_count(), count);
    }

    #[tokio::test]
    async fn another_track_in_the_queue() {
        let server = TestServer::start("group-playing").await;
        let provider = held(&server).await;
        server.patch_queue(
            json!({"current_item": {"queue_item_id": "another", "name": "Another Song"}}),
        );
        wait_for_claim(&provider, &stopped_on_its_own_queue(), false).await;
    }

    #[tokio::test]
    async fn polls_while_held() {
        let server = TestServer::start("group-playing").await;
        let _provider = held(&server).await;
        let players = server.requests("players/get").len();
        let queues = server.requests("player_queues/get").len();
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert!(server.requests("players/get").len() >= players + 2);
        assert!(server.requests("player_queues/get").len() >= queues + 2);
    }

    #[tokio::test]
    async fn a_stop_without_a_group_session_is_not_claimed() {
        let server = TestServer::start("group-paused").await;
        let provider = server.provider(TOKEN, FAST);
        let stopped = stopped_on_its_own_queue();
        provider.follow(&stopped);
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(!provider.claims(&stopped));
        assert_eq!(server.request_count(), 0);
    }
}

// Spec: session-music-assistant, "Settling the session".
mod settling {
    use super::*;

    /// Polls `settling` until it is `expected`, for at most `WAIT`.
    async fn wait_settling(
        provider: &jukelight_adapter_music_assistant::MusicAssistantProvider,
        device: &DeviceSnapshot,
        expected: bool,
    ) {
        let deadline = tokio::time::Instant::now() + WAIT;
        while provider.settling(device) != expected {
            assert!(
                tokio::time::Instant::now() < deadline,
                "settling never became {expected}"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    #[tokio::test]
    async fn airplay_no_answer_yet() {
        let server = TestServer::start("group-playing").await;
        let provider = server.provider(TOKEN, FAST);
        let device = airplay_device();
        provider.follow(&device);
        // Nothing asked yet: Music Assistant hasn't answered.
        assert!(provider.settling(&device));
    }

    #[tokio::test]
    async fn airplay_answered_drives_the_player() {
        let server = TestServer::start("group-playing").await;
        let provider = server.provider(TOKEN, FAST);
        let device = airplay_device();
        provider.follow(&device);
        wait_settling(&provider, &device, false).await;
        // Settled with its content: no "Playing from" in between.
        assert!(provider.claims(&device));
        assert!(provider.content().is_some());
    }

    #[tokio::test]
    async fn airplay_answered_a_phone() {
        let server = TestServer::start("airplay-phone").await;
        let provider = server.provider(TOKEN, FAST);
        let device = airplay_device();
        provider.follow(&device);
        wait_settling(&provider, &device, false).await;
        assert!(!provider.claims(&device));
    }

    #[tokio::test]
    async fn claimed_no_content_yet() {
        let server = TestServer::start("qobuz").await;
        let provider = server.provider(TOKEN, FAST);
        let device = claimed_device();
        provider.follow(&device);
        assert!(provider.settling(&device));
        wait_settling(&provider, &device, false).await;
        assert!(provider.content().is_some());
    }

    #[tokio::test]
    async fn unreachable() {
        let server = TestServer::start("group-playing").await;
        server.set_reachable(false);
        let provider = server.provider(TOKEN, FAST);
        let device = airplay_device();
        provider.follow(&device);
        wait_settling(&provider, &device, false).await;
        assert!(!provider.claims(&device));
    }

    #[tokio::test]
    async fn not_followed_never_settling() {
        let server = TestServer::start("group-playing").await;
        let provider = server.provider(TOKEN, FAST);
        let device = service_device();
        provider.follow(&device);
        assert!(!provider.settling(&device));
    }
}

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
