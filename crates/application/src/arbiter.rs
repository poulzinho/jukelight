use std::sync::Arc;

use jukelight_domain::{
    Content, DeviceSnapshot, InputKind, Origin, PlayerSnapshot, QueueView, Session, Track,
};

use crate::SessionProvider;

/// Builds the player snapshot from what the device reports and the first
/// session provider that claims the device's origin.
///
/// - A claiming provider with content supplies the content and the label
///   ("<label> · <service>" when the content names a music service).
/// - A claiming provider without content gives "Playing from <label>".
/// - Otherwise the device's own content is shown, labelled by its origin; an
///   audio input is labelled by its kind and carried on the snapshot.
///
/// Playback state and connectivity always come from the device.
pub fn arbitrate(
    device: &DeviceSnapshot,
    providers: &[Arc<dyn SessionProvider>],
) -> PlayerSnapshot {
    let claimant = providers.iter().find(|provider| provider.claims(device));

    let (session, content) = match claimant {
        Some(provider) => match provider.content() {
            Some(content) => (
                Session {
                    label: match &content.service {
                        Some(service) => format!("{} · {service}", provider.label()),
                        None => provider.label().into(),
                    },
                    available: true,
                },
                Some(content),
            ),
            None => (
                Session {
                    label: provider.label().into(),
                    available: false,
                },
                Some(unreachable(provider.label())),
            ),
        },
        None => (
            Session {
                label: origin_label(&device.origin),
                available: true,
            },
            device.content.clone(),
        ),
    };

    PlayerSnapshot {
        player_name: device.name.clone(),
        online: device.online,
        state: device.state,
        session,
        content,
        input: match &device.origin {
            Origin::Input(input) => Some(input.clone()),
            _ => None,
        },
    }
}

fn origin_label(origin: &Origin) -> String {
    match origin {
        Origin::Service { name, .. } => name.clone(),
        Origin::Stream { .. } => "Stream".into(),
        Origin::Input(input) => input_label(input.kind).into(),
        Origin::None => String::new(),
    }
}

fn input_label(kind: InputKind) -> &'static str {
    match kind {
        InputKind::Optical => "Optical input",
        InputKind::Analog => "Analog input",
        InputKind::HdmiArc => "HDMI ARC",
        InputKind::Other => "Input",
    }
}

/// What to show while the provider driving the device cannot be reached.
fn unreachable(label: &str) -> Content {
    Content {
        track: Track {
            title: format!("Playing from {label}"),
            artists: Vec::new(),
            album: None,
        },
        artwork: None,
        position: None,
        quality: None,
        queue: QueueView::NotApplicable,
        service: None,
        palette: None,
    }
}
