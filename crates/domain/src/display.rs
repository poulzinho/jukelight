use std::time::Duration;

use crate::{PlaybackState, PlayerSnapshot};

/// Whether the display should be on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayPower {
    On,
    Off,
}

/// When the display is on: while music plays, and for a grace period after
/// it stops. Knows no clock: the caller says how long the player has been
/// inactive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayPolicy {
    grace: Duration,
}

impl DisplayPolicy {
    pub fn new(grace: Duration) -> Self {
        Self { grace }
    }

    pub fn grace(&self) -> Duration {
        self.grace
    }

    /// Online and playing or buffering. Offline counts as inactive, whatever
    /// state the player last reported.
    pub fn is_active(player: &PlayerSnapshot) -> bool {
        player.online
            && matches!(
                player.state,
                PlaybackState::Playing | PlaybackState::Buffering
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
