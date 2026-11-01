use std::sync::Arc;
use std::time::Duration;

use jukelight_domain::{DisplayPolicy, DisplayPower, PlayerSnapshot, Timestamp};
use tokio::sync::watch;
use tokio::time::Instant;

use crate::Clock;

/// Applies the display policy over time: follows the player snapshots,
/// remembers since when the player is inactive and switches off when the
/// grace period ends. Waits with monotonic time, so clock changes cannot
/// switch the display; reads the wall clock only to know how long an audio
/// input's signal has been playing.
pub struct DisplayController {
    policy: DisplayPolicy,
    clock: Arc<dyn Clock>,
    power: watch::Sender<DisplayPower>,
}

/// Read access to the display decision, for the HTTP layer.
#[derive(Clone)]
pub struct DisplayHandle {
    power: watch::Receiver<DisplayPower>,
}

impl DisplayController {
    /// Starts on: the grace period counts from the start.
    pub fn new(policy: DisplayPolicy, clock: Arc<dyn Clock>) -> Self {
        let (power, _) = watch::channel(DisplayPower::On);
        Self {
            policy,
            clock,
            power,
        }
    }

    pub fn handle(&self) -> DisplayHandle {
        DisplayHandle {
            power: self.power.subscribe(),
        }
    }

    /// Follows `players` (published snapshots) until the channel closes.
    pub async fn run(self, mut players: watch::Receiver<PlayerSnapshot>) {
        let mut inactive_since = Some(Instant::now());
        loop {
            let player = players.borrow_and_update().clone();
            let now = self.clock.now();
            let active = self.policy.is_active(&player, now);
            inactive_since = if active {
                None
            } else {
                // An input past its limit became inactive when the limit
                // passed, which may be before this snapshot arrived.
                let became = self.became_inactive(&player, now);
                Some(inactive_since.map_or(became, |since| since.min(became)))
            };
            let inactive_for = inactive_since.map(|since| since.elapsed());
            let decision = self.policy.decide(inactive_for);
            self.power.send_if_modified(|current| {
                let changed = *current != decision;
                *current = decision;
                changed
            });

            // Wake up when the grace period ends, or when an input that
            // still counts as playing reaches its time limit.
            let grace_ends = match decision {
                DisplayPower::On => self.policy.remaining(inactive_for),
                DisplayPower::Off => None,
            };
            // Never a zero wait: it would spin without letting time pass.
            let input_ends = if active {
                self.policy
                    .input_time_left(&player, now)
                    .filter(|left| !left.is_zero())
            } else {
                None
            };
            let wait = grace_ends.into_iter().chain(input_ends).min();
            tokio::select! {
                changed = players.changed() => if changed.is_err() { return },
                () = sleep(wait) => {}
            }
        }
    }

    /// When the player stopped counting as active: now, or for an input
    /// past its time limit, when the limit passed.
    fn became_inactive(&self, player: &PlayerSnapshot, now: Timestamp) -> Instant {
        let overdue = player
            .input
            .as_ref()
            .and_then(|input| input.started_at)
            .filter(|_| self.policy.input_time_left(player, now) == Some(Duration::ZERO))
            .map(|started_at| {
                now.since(started_at)
                    .saturating_sub(self.policy.input_limit())
            });
        let at = Instant::now();
        overdue
            .and_then(|overdue| at.checked_sub(overdue))
            .unwrap_or(at)
    }
}

async fn sleep(duration: Option<Duration>) {
    match duration {
        Some(duration) => tokio::time::sleep(duration).await,
        None => std::future::pending().await,
    }
}

impl DisplayHandle {
    pub fn current(&self) -> DisplayPower {
        *self.power.borrow()
    }

    /// Notifies on every new decision; starts with the current one.
    pub fn subscribe(&self) -> watch::Receiver<DisplayPower> {
        self.power.clone()
    }
}
