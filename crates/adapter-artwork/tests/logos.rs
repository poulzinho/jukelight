//! Spec: published-api, "Brand logos": the folder the user supplies logos in.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use jukelight_adapter_artwork::LogoFolder;
use jukelight_application::BrandLogos;
use jukelight_domain::Brand;

/// An empty folder of its own, removed when the test ends.
struct Folder(PathBuf);

impl Folder {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jukelight-logos-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, bytes: &[u8]) {
        std::fs::write(self.0.join(name), bytes).unwrap();
    }

    /// Re-scanned on every request, so changes show at once.
    fn logos(&self) -> LogoFolder {
        LogoFolder::new(&self.0, Duration::ZERO)
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn brand(id: &str) -> Brand {
    Brand::new(id).unwrap()
}

#[test]
fn svg_logo() {
    let folder = Folder::new();
    folder.write("qobuz.svg", b"<svg/>");
    let logos = folder.logos();
    assert!(logos.version(&brand("qobuz")).is_some());
    let logo = logos.logo(&brand("qobuz")).unwrap();
    assert_eq!(logo.content_type, "image/svg+xml");
    assert_eq!(logo.bytes, b"<svg/>");
}

#[test]
fn png_logo_and_svg_first() {
    let folder = Folder::new();
    folder.write("bluesound.png", b"png");
    assert_eq!(
        folder
            .logos()
            .logo(&brand("bluesound"))
            .unwrap()
            .content_type,
        "image/png"
    );
    folder.write("bluesound.svg", b"<svg/>");
    assert_eq!(
        folder
            .logos()
            .logo(&brand("bluesound"))
            .unwrap()
            .content_type,
        "image/svg+xml"
    );
}

#[test]
fn no_logo() {
    let folder = Folder::new();
    assert_eq!(folder.logos().version(&brand("qobuz")), None);
    assert_eq!(folder.logos().logo(&brand("qobuz")), None);
}

#[test]
fn logo_added_and_removed_later() {
    let folder = Folder::new();
    let logos = folder.logos();
    assert_eq!(logos.version(&brand("spotify")), None);
    folder.write("spotify.svg", b"<svg/>");
    assert!(logos.version(&brand("spotify")).is_some());
    std::fs::remove_file(folder.0.join("spotify.svg")).unwrap();
    assert_eq!(logos.version(&brand("spotify")), None);
}

#[test]
fn replaced_logo_gets_a_new_version() {
    let folder = Folder::new();
    folder.write("tidal.svg", b"<svg/>");
    let logos = folder.logos();
    let first = logos.version(&brand("tidal")).unwrap();
    std::thread::sleep(Duration::from_millis(1100));
    folder.write("tidal.svg", b"<svg id='new'/>");
    assert_ne!(logos.version(&brand("tidal")).unwrap(), first);
}

#[test]
fn missing_folder_means_no_logos() {
    let logos = LogoFolder::new(
        std::path::Path::new("/nonexistent/brand-assets"),
        Duration::ZERO,
    );
    assert_eq!(logos.version(&brand("qobuz")), None);
}

#[test]
fn results_are_cached_for_a_while() {
    let folder = Folder::new();
    let logos = LogoFolder::new(&folder.0, Duration::from_secs(60));
    assert_eq!(logos.version(&brand("deezer")), None);
    folder.write("deezer.svg", b"<svg/>");
    assert_eq!(
        logos.version(&brand("deezer")),
        None,
        "still the cached answer"
    );
}
