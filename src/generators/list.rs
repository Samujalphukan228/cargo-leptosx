//! `list` — show the project's installed features from leptosx.toml.

use crate::generators::{self, ALL_FEATURES};
use anyhow::Result;

pub fn run() -> Result<()> {
    generators::ensure_project()?;
    let doc = generators::leptosx_toml()?;

    println!(
        "project:   {}",
        doc["project"]["name"].as_str().unwrap_or("?")
    );
    println!(
        "rendering: {}",
        doc["project"]["rendering"].as_str().unwrap_or("?")
    );
    println!("features:");

    let mut any = false;
    for feature in ALL_FEATURES {
        let key = generators::toml_key(feature).unwrap();
        if generators::feature_installed(key).unwrap_or(false) {
            any = true;
            println!("  ✓ {feature}");
        }
    }
    if !any {
        println!("  (none)");
    }
    Ok(())
}