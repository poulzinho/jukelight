//! Spec: the source's brand (DEV-33): brands, services and sources.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use jukelight_domain::{Brand, Route, Service, Source};

#[test]
fn brands_are_short_lowercase_identifiers() {
    for id in ["qobuz", "apple-music", "hi-res", "bluesound"] {
        assert_eq!(Brand::new(id).unwrap().as_str(), id);
    }
    // They double as logo file names, so nothing that could leave the folder.
    for id in ["", "Qobuz", "../etc", "qobuz.svg", "a b", "-x"] {
        assert_eq!(Brand::new(id), None, "{id:?}");
    }
}

#[test]
fn a_service_has_a_brand_and_a_name() {
    let service = Service::new(Brand::new("qobuz"), "My Qobuz");
    assert_eq!(service.brand.as_ref().map(Brand::as_str), Some("qobuz"));
    assert_eq!(service.name, "My Qobuz");
}

#[test]
fn a_source_names_its_route() {
    let source = Source {
        service: Service::new(Brand::new("qobuz"), "Qobuz"),
        via: Some(Route {
            service: Service::new(Brand::new("music-assistant"), "Music Assistant"),
            available: false,
        }),
    };
    assert!(!source.via.as_ref().unwrap().available);
}
