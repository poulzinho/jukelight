use std::time::Duration;

use crate::{PlaybackState, PlayerSnapshot, Timestamp};

/// Whether the display should be on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayPower {
    On,
    Off,
}

/// Inputs count as playing for this long after their signal started, unless
/// configured otherwise: a full CD (at most 80 minutes) and some slack.
pub const DEFAULT_INPUT_LIMIT: Duration = Duration::from_secs(90 * 60);

/// When the display is on: while music plays, and for a grace period after
/// it stops. An audio input counts as playing only once its signal started,
/// and then for a limited time: the player notices when a signal starts but
/// not when it stops. Knows no timer: the caller says the time and how long
/// the player has been inactive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayPolicy {
    grace: Duration,
    input_limit: Duration,
}

impl DisplayPolicy {
    pub fn new(grace: Duration) -> Self {
        Self {
            grace,
            input_limit: DEFAULT_INPUT_LIMIT,
        }
    }

    #[must_use]
    pub fn with_input_limit(mut self, limit: Duration) -> Self {
        self.input_limit = limit;
        self
    }

    pub fn grace(&self) -> Duration {
        self.grace
    }

    pub fn input_limit(&self) -> Duration {
        self.input_limit
    }

    /// Online and playing or buffering; for an input, only once its signal
    /// started and within its time limit. Offline counts as inactive,
    /// whatever state the player last reported.
    pub fn is_active(&self, player: &PlayerSnapshot, now: Timestamp) -> bool {
        player.online
            && matches!(
                player.state,
                PlaybackState::Playing | PlaybackState::Buffering
            )
            && match &player.input {
                Some(input) => {
                    input.started_at.is_some()
                        && self.input_time_left(player, now) != Some(Duration::ZERO)
                }
                None => true,
            }
    }

    /// How long an input whose signal started still counts as playing
    /// (`None` without an input or before its signal started).
    pub fn input_time_left(&self, player: &PlayerSnapshot, now: Timestamp) -> Option<Duration> {
        let started_at = player.input.as_ref()?.started_at?;
        Some(self.input_limit.saturating_sub(now.since(started_at)))
    }

    /// The decision, given how long the player has been inactive (`None`
    /// while it is active).
    pub fn decide(&self, inactive_for: Option<Duration>) -> DisplayPower {
        match inactive_for {
            Some(inactive) if inactive >= self.grace => DisplayPower::Off,
            _ => DisplayPower::On,
        }
    }

    /// How long until the display goes off (`None` while the player is
    /// active).
    pub fn remaining(&self, inactive_for: Option<Duration>) -> Option<Duration> {
        inactive_for.map(|inactive| self.grace.saturating_sub(inactive))
    }
}
