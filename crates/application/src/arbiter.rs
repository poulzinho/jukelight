use std::sync::Arc;

use jukelight_domain::{
    Content, DeviceSnapshot, InputKind, Origin, PlayerSnapshot, QueueView, Route, Service, Session,
    Source, Track,
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
/// The source names the music service and the route it takes: a claiming
/// provider's service via the provider (the provider alone when it names no
/// service; marked unavailable when it can't be reached), or the device's own
/// service or input without a route.
///
/// Playback state, connectivity and the player's maker always come from the
/// device.
pub fn arbitrate(
    device: &DeviceSnapshot,
    providers: &[Arc<dyn SessionProvider>],
) -> PlayerSnapshot {
    let claimant = providers.iter().find(|provider| provider.claims(device));

    let (session, content, source) = match claimant {
        Some(provider) => {
            let itself = Service::new(provider.brand(), provider.label());
            match provider.content() {
                Some(content) => {
                    let source = Source {
                        via: content.service.as_ref().map(|_| Route {
                            service: itself.clone(),
                            available: true,
                        }),
                        service: content.service.clone().unwrap_or(itself),
                    };
                    (
                        Session {
                            label: match &content.service {
                                Some(service) => {
                                    format!("{} · {}", provider.label(), service.name)
                                }
                                None => provider.label().into(),
                            },
                            available: true,
                        },
                        Some(content),
                        Some(source),
                    )
                }
                None => (
                    Session {
                        label: provider.label().into(),
                        available: false,
                    },
                    Some(unreachable(provider.label())),
                    Some(Source {
                        service: itself.clone(),
                        via: Some(Route {
                            service: itself,
                            available: false,
                        }),
                    }),
                ),
            }
        }
        None => (
            Session {
                label: origin_label(&device.origin),
                available: true,
            },
            device.content.clone(),
            origin_source(&device.origin),
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
        source,
        maker: device.maker.clone(),
    }
}

/// The device's own source, without a route.
fn origin_source(origin: &Origin) -> Option<Source> {
    let service = match origin {
        Origin::Service { name, brand, .. } => Service::new(brand.clone(), name.clone()),
        Origin::Stream { .. } | Origin::Input(_) => Service::new(None, origin_label(origin)),
        Origin::None => return None,
    };
    Some(Source { service, via: None })
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
        InputKind::Analog => "Analogue input",
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
