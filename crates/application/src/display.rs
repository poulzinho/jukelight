use jukelight_domain::{DisplayPolicy, DisplayPower, PlayerSnapshot};
use tokio::sync::watch;
use tokio::time::Instant;

/// Applies the display policy over time: follows the player snapshots,
/// remembers since when the player is inactive and switches off when the
/// grace period ends. Uses monotonic time, so clock changes cannot switch
/// the display.
pub struct DisplayController {
    policy: DisplayPolicy,
    power: watch::Sender<DisplayPower>,
}

/// Read access to the display decision, for the HTTP layer.
#[derive(Clone)]
pub struct DisplayHandle {
    power: watch::Receiver<DisplayPower>,
}

impl DisplayController {
    /// Starts on: the grace period counts from the start.
    pub fn new(policy: DisplayPolicy) -> Self {
        let (power, _) = watch::channel(DisplayPower::On);
        Self { policy, power }
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
            let active = DisplayPolicy::is_active(&players.borrow_and_update());
            inactive_since = match (active, inactive_since) {
                (true, _) => None,
                (false, None) => Some(Instant::now()),
                (false, since) => since,
            };
            let inactive_for = inactive_since.map(|since| since.elapsed());
            let decision = self.policy.decide(inactive_for);
            self.power.send_if_modified(|current| {
                let changed = *current != decision;
                *current = decision;
                changed
            });

            // While on and inactive, wake up when the grace period ends.
            let deadline = match decision {
                DisplayPower::On => self
                    .policy
                    .remaining(inactive_for)
                    .map(|remaining| Instant::now() + remaining),
                DisplayPower::Off => None,
            };
            tokio::select! {
                changed = players.changed() => if changed.is_err() { return },
                () = sleep_until(deadline) => {}
            }
        }
    }
}

async fn sleep_until(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
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
