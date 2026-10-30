//! Copies the built kiosk (`web/kiosk/dist`) into the build output, where
//! `rust-embed` picks it up. Without a kiosk build, a placeholder page is
//! embedded instead, so Rust-only builds and tests still work.

use std::path::{Path, PathBuf};
use std::{env, fs, io};

const PLACEHOLDER: &str = "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><title>Jukelight</title></head>\n<body><p>The kiosk has not been built. Run <code>npm run build</code> in <code>web/kiosk</code>.</p></body></html>\n";

fn main() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(io::Error::other)?);
    let dist = manifest_dir.join("../../web/kiosk/dist");
    let target = PathBuf::from(env::var("OUT_DIR").map_err(io::Error::other)?).join("kiosk");

    println!("cargo::rerun-if-changed={}", dist.display());
    println!("cargo::rerun-if-changed=build.rs");

    if target.exists() {
        fs::remove_dir_all(&target)?;
    }
    if dist.join("index.html").is_file() {
        copy_dir(&dist, &target)
    } else {
        println!("cargo::warning=web/kiosk/dist not found; embedding a placeholder kiosk page");
        fs::create_dir_all(&target)?;
        fs::write(target.join("index.html"), PLACEHOLDER)
    }
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let destination = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &destination)?;
        } else {
            fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}
