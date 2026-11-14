//! Spec: artwork, against a local test server.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use jukelight_adapter_artwork::ArtworkProxy;
use jukelight_application::{ArtworkError, ArtworkId, ArtworkSize, ArtworkSource};
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

    let image = proxy.fetch(&id, ArtworkSize::Original).await.unwrap();
    assert_eq!(image.content_type, "image/jpeg");
    assert_eq!(image.bytes, JPEG);
}

#[tokio::test]
async fn unknown_id_makes_no_request() {
    let (_, hits) = source().await;
    let proxy = ArtworkProxy::new();

    let result = proxy
        .fetch(&ArtworkId::new("0000000000000000"), ArtworkSize::Original)
        .await;
    assert_eq!(result, Err(ArtworkError::Unknown));
    assert_eq!(hits.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn redirecting_source() {
    let (base, _) = source().await;
    let proxy = ArtworkProxy::new();
    let id = proxy.issue(&ArtworkRef::new(format!("{base}/moved")));

    assert_eq!(
        proxy.fetch(&id, ArtworkSize::Original).await.unwrap().bytes,
        JPEG
    );
}

#[tokio::test]
async fn source_fails() {
    let (base, _) = source().await;
    let proxy = ArtworkProxy::new();
    let broken = proxy.issue(&ArtworkRef::new(format!("{base}/broken")));
    let unreachable = proxy.issue(&ArtworkRef::new("http://127.0.0.1:1/cover.jpg"));

    assert!(matches!(
        proxy.fetch(&broken, ArtworkSize::Original).await,
        Err(ArtworkError::Source(_))
    ));
    assert!(matches!(
        proxy.fetch(&unreachable, ArtworkSize::Original).await,
        Err(ArtworkError::Source(_))
    ));
}

#[tokio::test]
async fn second_request_uses_the_cache() {
    let (base, hits) = source().await;
    let proxy = ArtworkProxy::new();
    let id = proxy.issue(&ArtworkRef::new(format!("{base}/cover.jpg")));

    proxy.fetch(&id, ArtworkSize::Original).await.unwrap();
    proxy.fetch(&id, ArtworkSize::Original).await.unwrap();
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
        proxy.fetch(id, ArtworkSize::Original).await.unwrap();
    }
    // The first image was evicted, so fetching it contacts the source again.
    proxy.fetch(&ids[0], ArtworkSize::Original).await.unwrap();
    assert_eq!(hits.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn data_url_source() {
    let proxy = ArtworkProxy::new();
    // "<svg/>" in base64.
    let id = proxy.issue(&ArtworkRef::new("data:image/svg+xml;base64,PHN2Zy8+"));

    let image = proxy.fetch(&id, ArtworkSize::Original).await.unwrap();
    assert_eq!(image.content_type, "image/svg+xml");
    assert_eq!(image.bytes, b"<svg/>");
}

// Spec: artwork, "Large covers". The CDNs' hosts point at a local test
// server; the URL rules themselves are pinned by the unit tests in lib.rs.

const SMALL: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 6, 0, 0];
const LARGE: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 9, 9, 9];
const SPOTIFY_ID: &str = "1234567890abcdef12345678";

/// A stand-in for the Qobuz and Spotify CDNs, and a player that redirects
/// to them. Returns its address as `host:port`.
async fn cdn() -> String {
    let jpeg = |bytes: &'static [u8]| {
        move || async move { ([(header::CONTENT_TYPE, "image/jpeg")], bytes).into_response() }
    };
    let app = Router::new()
        .route("/images/covers/ab/cd/abcdef_600.jpg", get(jpeg(SMALL)))
        .route("/images/covers/ab/cd/abcdef_max.jpg", get(jpeg(LARGE)))
        .route("/images/covers/ef/gh/nomax_600.jpg", get(jpeg(SMALL)))
        .route(
            &format!("/image/ab67616d0000b273{SPOTIFY_ID}"),
            get(jpeg(SMALL)),
        )
        .route(
            &format!("/image/ab67616d000082c1{SPOTIFY_ID}"),
            get(jpeg(LARGE)),
        )
        .route("/plain.jpg", get(jpeg(SMALL)))
        .route(
            "/Artwork",
            get(|| async { Redirect::temporary("/images/covers/ab/cd/abcdef_600.jpg") }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let host = format!("127.0.0.1:{}", listener.local_addr().unwrap().port());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    host
}

async fn large(host: &str, path: &str) -> Vec<u8> {
    let proxy = ArtworkProxy::new().with_cdn_hosts(host, host);
    let id = proxy.issue(&ArtworkRef::new(format!("http://{host}{path}")));
    proxy.fetch(&id, ArtworkSize::Large).await.unwrap().bytes
}

#[tokio::test]
async fn qobuz_cover_after_a_redirect() {
    let host = cdn().await;
    assert_eq!(large(&host, "/Artwork?songid=1").await, LARGE);
}

#[tokio::test]
async fn spotify_cover() {
    let host = cdn().await;
    assert_eq!(
        large(&host, &format!("/image/ab67616d0000b273{SPOTIFY_ID}")).await,
        LARGE
    );
}

#[tokio::test]
async fn larger_version_missing() {
    let host = cdn().await;
    assert_eq!(
        large(&host, "/images/covers/ef/gh/nomax_600.jpg").await,
        SMALL
    );
}

#[tokio::test]
async fn unknown_source() {
    let host = cdn().await;
    assert_eq!(large(&host, "/plain.jpg").await, SMALL);
}

#[tokio::test]
async fn original_stays_original() {
    let host = cdn().await;
    let proxy = ArtworkProxy::new().with_cdn_hosts(&host, &host);
    let id = proxy.issue(&ArtworkRef::new(format!("http://{host}/Artwork?songid=1")));
    assert_eq!(
        proxy.fetch(&id, ArtworkSize::Original).await.unwrap().bytes,
        SMALL
    );
}

// Spec: artwork, "Backdrop image".

/// A 600 by 600 pixel PNG.
fn cover_png() -> Vec<u8> {
    let cover = image::RgbImage::from_fn(600, 600, |x, y| {
        image::Rgb([
            u8::try_from(x % 256).unwrap(),
            u8::try_from(y % 256).unwrap(),
            128,
        ])
    });
    let mut png = Vec::new();
    cover
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .unwrap();
    png
}

async fn serve_once(content_type: &'static str, bytes: Vec<u8>) -> String {
    let app = Router::new().route(
        "/cover",
        get(move || {
            let bytes = bytes.clone();
            async move { ([(header::CONTENT_TYPE, content_type)], bytes).into_response() }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!(
        "http://127.0.0.1:{}/cover",
        listener.local_addr().unwrap().port()
    );
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    url
}

#[tokio::test]
async fn backdrop_of_a_cover() {
    let url = serve_once("image/png", cover_png()).await;
    let proxy = ArtworkProxy::new();
    let id = proxy.issue(&ArtworkRef::new(url));

    let backdrop = proxy.fetch(&id, ArtworkSize::Backdrop).await.unwrap();
    assert_eq!(backdrop.content_type, "image/png");
    let decoded = image::load_from_memory(&backdrop.bytes).unwrap();
    assert!(
        decoded.width() <= 128 && decoded.height() <= 128 && decoded.width() > 32,
        "{}x{}",
        decoded.width(),
        decoded.height()
    );
}

fn png_of(cover: &image::RgbImage) -> Vec<u8> {
    let mut png = Vec::new();
    cover
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .unwrap();
    png
}

async fn backdrop_of(cover: &image::RgbImage) -> image::RgbImage {
    let url = serve_once("image/png", png_of(cover)).await;
    let proxy = ArtworkProxy::new();
    let id = proxy.issue(&ArtworkRef::new(url));
    let backdrop = proxy.fetch(&id, ArtworkSize::Backdrop).await.unwrap();
    image::load_from_memory(&backdrop.bytes).unwrap().to_rgb8()
}

/// Spec: artwork, "Blurred": no sharp edge where the cover has one.
#[tokio::test]
async fn blurred() {
    let cover = image::RgbImage::from_fn(600, 600, |x, _| {
        if x < 300 {
            image::Rgb([0, 0, 0])
        } else {
            image::Rgb([255, 255, 255])
        }
    });
    let backdrop = backdrop_of(&cover).await;
    let y = backdrop.height() / 2;
    let row: Vec<i32> = (0..backdrop.width())
        .map(|x| i32::from(backdrop.get_pixel(x, y)[0]))
        .collect();
    let steepest = row.windows(2).map(|w| (w[1] - w[0]).abs()).max().unwrap();
    assert!(
        steepest <= 40,
        "a sharp edge: {steepest} levels between neighbours"
    );
    // Still dark on the left and light on the right.
    assert!(row[2] < 80 && row[row.len() - 3] > 175, "{row:?}");
}

/// Spec: artwork, "Grain against banding": a flat colour gets a faint grain.
#[tokio::test]
async fn grain_against_banding() {
    let cover = image::RgbImage::from_pixel(600, 600, image::Rgb([60, 70, 80]));
    let backdrop = backdrop_of(&cover).await;
    let values: Vec<i32> = backdrop.pixels().map(|p| i32::from(p[0])).collect();
    let (min, max) = (values.iter().min().unwrap(), values.iter().max().unwrap());
    assert!(max > min, "no grain at all");
    assert!(max - min <= 8, "grain too strong: {min}..{max}");
}

/// The same cover always gives the same backdrop.
#[tokio::test]
async fn the_same_cover_gives_the_same_backdrop() {
    let cover = image::RgbImage::from_pixel(600, 600, image::Rgb([60, 70, 80]));
    assert_eq!(backdrop_of(&cover).await, backdrop_of(&cover).await);
}

#[tokio::test]
async fn backdrop_of_something_that_is_not_an_image() {
    let url = serve_once("image/jpeg", JPEG.to_vec()).await;
    let proxy = ArtworkProxy::new();
    let id = proxy.issue(&ArtworkRef::new(url));

    assert!(matches!(
        proxy.fetch(&id, ArtworkSize::Backdrop).await,
        Err(ArtworkError::Source(_))
    ));
}
