//! Logos the user supplies in a folder, by brand: `<brand>.svg` or
//! `<brand>.png`. The project ships none; without a file, there is no logo.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant, UNIX_EPOCH};

use jukelight_application::{BrandLogos, Image};
use jukelight_domain::Brand;

/// File types in the order they are looked for.
const TYPES: [(&str, &str); 2] = [("svg", "image/svg+xml"), ("png", "image/png")];

/// The user's logo folder. Lookups are cached for `fresh`, so files added,
/// replaced or removed show up within that time, without a restart.
pub struct LogoFolder {
    folder: PathBuf,
    fresh: Duration,
    found: Mutex<HashMap<String, (Instant, Option<Found>)>>,
}

#[derive(Clone)]
struct Found {
    path: PathBuf,
    content_type: &'static str,
    version: u64,
}

impl LogoFolder {
    pub fn new(folder: &Path, fresh: Duration) -> Self {
        Self {
            folder: folder.to_owned(),
            fresh,
            found: Mutex::new(HashMap::new()),
        }
    }

    fn find(&self, brand: &Brand) -> Option<Found> {
        let mut found = self.found.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((at, logo)) = found.get(brand.as_str())
            && at.elapsed() < self.fresh
        {
            return logo.clone();
        }
        let logo = self.look_up(brand);
        found.insert(brand.as_str().to_owned(), (Instant::now(), logo.clone()));
        logo
    }

    /// The brand's file, if there is one. A brand is lowercase letters,
    /// digits and hyphens, so the path stays inside the folder.
    fn look_up(&self, brand: &Brand) -> Option<Found> {
        TYPES.iter().find_map(|(extension, content_type)| {
            let path = self.folder.join(format!("{}.{extension}", brand.as_str()));
            let metadata = std::fs::metadata(&path)
                .ok()
                .filter(std::fs::Metadata::is_file)?;
            Some(Found {
                version: version(&metadata),
                path,
                content_type,
            })
        })
    }
}

/// Changes when the file is replaced: its modification time and size.
fn version(metadata: &std::fs::Metadata) -> u64 {
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |since| {
            u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
        });
    modified
        .wrapping_mul(1_000_003)
        .wrapping_add(metadata.len())
}

impl BrandLogos for LogoFolder {
    fn version(&self, brand: &Brand) -> Option<u64> {
        self.find(brand).map(|found| found.version)
    }

    fn logo(&self, brand: &Brand) -> Option<Image> {
        let found = self.find(brand)?;
        let bytes = std::fs::read(&found.path).ok()?;
        Some(Image {
            content_type: found.content_type.to_owned(),
            bytes,
        })
    }
}
