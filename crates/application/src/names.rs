use std::collections::HashMap;

use jukelight_domain::{Output, OutputKind, PlayerSnapshot};

/// Your names for speakers and for the speaker on the player's output, from
/// configuration. The player can't know what is plugged into its output;
/// Music Assistant's names may be a computer's or a model's.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpeakerNames {
    /// From the name a speaker reports to your name for it.
    names: HashMap<String, String>,
    /// The speaker on the player's output.
    output: Option<String>,
}

impl SpeakerNames {
    pub fn new(names: HashMap<String, String>, output: Option<String>) -> Self {
        Self { names, output }
    }

    /// Renames the session's speakers and puts the configured output speaker
    /// before the outputs the device reports.
    pub fn apply(&self, mut snapshot: PlayerSnapshot) -> PlayerSnapshot {
        if let Some(content) = snapshot.content.as_mut() {
            for speaker in &mut content.speakers {
                if let Some(name) = self.names.get(&speaker.name) {
                    speaker.name.clone_from(name);
                }
            }
        }
        if let Some(name) = &self.output {
            snapshot.outputs.insert(
                0,
                Output {
                    name: Some(name.clone()),
                    kind: OutputKind::Speaker,
                },
            );
        }
        snapshot
    }
}
