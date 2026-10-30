//! Spec: artwork, against a local test server.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use jukelight_adapter_artwork::ArtworkProxy;
use jukelight_application::{ArtworkError, ArtworkId, ArtworkSource};
use jukelight_domain::ArtworkRef;

const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 1, 2, 3];

/// A source of images that counts the requests it receives.
async fn source() -> (String, Arc<AtomicUsize>) {
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&hits);
    let app = Router::new()
        .route(
            "/cover.jpg",
            get(move || {
                counter.fetch_add(1, Ordering::SeqCst);
                async { ([(header::CONTENT_TYPE, "image/jpeg")], JPEG).into_response() }
            }),
        )
        .route(
            "/moved",
            get(|| async { Redirect::temporary("/cover.jpg") }),
        )
        .route(
            "/broken",
            get(|| async { StatusCode::INTERNAL_SERVER_ERROR }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (base, hits)
}

// 6.1 Opaque artwork ids

#[test]
fn same_source_same_id() {
    let proxy = ArtworkProxy::new();
    let source = ArtworkRef::new("http://192.168.1.40:11000/Artwork?songid=1");
    assert_eq!(proxy.issue(&source), proxy.issue(&source));
}

#[test]
fn different_sources_different_ids() {
    let proxy = ArtworkProxy::new();
    let first = proxy.issue(&ArtworkRef::new(
        "http://192.168.1.40:11000/Artwork?songid=1",
    ));
    let second = proxy.issue(&ArtworkRef::new(
        "http://192.168.1.40:11000/Artwork?songid=2",
    ));
    assert_ne!(first, second);
}

#[test]
fn id_reveals_nothing_of_the_source() {
    let proxy = ArtworkProxy::new();
    let id = proxy.issue(&ArtworkRef::new(
        "http://192.168.1.40:11000/Artwork?songid=1",
    ));
    assert_eq!(id.as_str().len(), 16);
    assert!(id.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    assert!(!id.as_str().contains("192"));
}

// 6.2 Artwork proxy

#[tokio::test]
async fn issued_id_returns_the_image() {
    let (base, _) = source().await;
    let proxy = ArtworkProxy::new();
    let id = proxy.issue(&ArtworkRef::new(format!("{base}/cover.jpg")));

    let image = proxy.fetch(&id).await.unwrap();
    assert_eq!(image.content_type, "image/jpeg");
    assert_eq!(image.bytes, JPEG);
}

#[tokio::test]
async fn unknown_id_makes_no_request() {
    let (_, hits) = source().await;
    let proxy = ArtworkProxy::new();

    let result = proxy.fetch(&ArtworkId::new("0000000000000000")).await;
    assert_eq!(result, Err(ArtworkError::Unknown));
    assert_eq!(hits.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn redirecting_source() {
    let (base, _) = source().await;
    let proxy = ArtworkProxy::new();
    let id = proxy.issue(&ArtworkRef::new(format!("{base}/moved")));

    assert_eq!(proxy.fetch(&id).await.unwrap().bytes, JPEG);
}

#[tokio::test]
async fn source_fails() {
    let (base, _) = source().await;
    let proxy = ArtworkProxy::new();
    let broken = proxy.issue(&ArtworkRef::new(format!("{base}/broken")));
    let unreachable = proxy.issue(&ArtworkRef::new("http://127.0.0.1:1/cover.jpg"));

    assert!(matches!(
        proxy.fetch(&broken).await,
        Err(ArtworkError::Source(_))
    ));
    assert!(matches!(
        proxy.fetch(&unreachable).await,
        Err(ArtworkError::Source(_))
    ));
}

#[tokio::test]
async fn second_request_uses_the_cache() {
    let (base, hits) = source().await;
    let proxy = ArtworkProxy::new();
    let id = proxy.issue(&ArtworkRef::new(format!("{base}/cover.jpg")));

    proxy.fetch(&id).await.unwrap();
    proxy.fetch(&id).await.unwrap();
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn cache_is_bounded() {
    let (base, hits) = source().await;
    let proxy = ArtworkProxy::with_capacity(2);
    let ids: Vec<_> = (0..3)
        .map(|n| proxy.issue(&ArtworkRef::new(format!("{base}/cover.jpg?n={n}"))))
        .collect();
    for id in &ids {
        proxy.fetch(id).await.unwrap();
    }
    // The first image was evicted, so fetching it contacts the source again.
    proxy.fetch(&ids[0]).await.unwrap();
    assert_eq!(hits.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn data_url_source() {
    let proxy = ArtworkProxy::new();
    // "<svg/>" in base64.
    let id = proxy.issue(&ArtworkRef::new("data:image/svg+xml;base64,PHN2Zy8+"));

    let image = proxy.fetch(&id).await.unwrap();
    assert_eq!(image.content_type, "image/svg+xml");
    assert_eq!(image.bytes, b"<svg/>");
}
