use std::time::Duration;

use crate::{PlaybackState, PlayerSnapshot, Timestamp};

/// Whether the display should be on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayPower {
    On,
    Off,
}

/// Inputs count as playing for this long after they were selected, unless
/// configured otherwise: a full CD (at most 80 minutes) and some slack.
pub const DEFAULT_INPUT_LIMIT: Duration = Duration::from_secs(90 * 60);

/// When the display is on: while music plays, and for a grace period after
/// it stops. An audio input counts as playing only for a limited time after
/// it was selected: the player cannot tell whether a signal comes in. Knows
/// no timer: the caller says the time and how long the player has been
/// inactive.
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

    /// Online and playing or buffering, and for an input within its time
    /// limit. Offline counts as inactive, whatever state the player last
    /// reported.
    pub fn is_active(&self, player: &PlayerSnapshot, now: Timestamp) -> bool {
        player.online
            && matches!(
                player.state,
                PlaybackState::Playing | PlaybackState::Buffering
            )
            && self.input_time_left(player, now) != Some(Duration::ZERO)
    }

    /// How long an input still counts as playing (`None` without an input).
    pub fn input_time_left(&self, player: &PlayerSnapshot, now: Timestamp) -> Option<Duration> {
        let input = player.input.as_ref()?;
        Some(
            self.input_limit
                .saturating_sub(now.since(input.selected_at)),
        )
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
