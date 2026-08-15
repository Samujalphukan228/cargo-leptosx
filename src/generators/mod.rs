//! Feature management shared by `add`, `remove`, `list`, and `doctor`.

pub mod add;
pub mod doctor;
pub mod list;
pub mod new;
pub mod pages_scan;
pub mod remove;

use anyhow::{bail, Result};
use std::fs;
use toml_edit::{value, Document};

pub const ALL_FEATURES: &[&str] = &["tailwind", "router", "lucide", "ui", "api", "file-router"];

/// Map a CLI feature name to its key in `leptosx.toml` under `[features]`.
pub fn toml_key(feature: &str) -> Option<&str> {
    match feature {
        "tailwind" => Some("tailwind"),
        "router" => Some("router"),
        "lucide" => Some("lucide"),
        "ui" => Some("component_lib"),
        "api" => Some("api"),
        "file-router" => Some("file_router"),
        _ => None,
    }
}

/// Reject unknown feature names with a helpful message.
pub fn validate_feature(feature: &str) -> Result<()> {
    if !ALL_FEATURES.contains(&feature) {
        bail!(
            "unknown feature `{feature}` (try: {})",
            ALL_FEATURES.join(", ")
        );
    }
    Ok(())
}

pub fn ensure_project() -> Result<()> {
    if fs::metadata("leptosx.toml").is_err() {
        bail!(
            "no leptosx.toml found — run this inside a project created with `cargo leptosx new`"
        );
    }
    Ok(())
}

pub fn cargo_toml() -> Result<Document> {
    Ok(fs::read_to_string("Cargo.toml")?.parse::<Document>()?)
}

pub fn write_cargo_toml(doc: &Document) -> Result<()> {
    fs::write("Cargo.toml", doc.to_string())?;
    Ok(())
}

pub fn leptosx_toml() -> Result<Document> {
    Ok(fs::read_to_string("leptosx.toml")?.parse::<Document>()?)
}

pub fn write_leptosx_toml(doc: &Document) -> Result<()> {
    fs::write("leptosx.toml", doc.to_string())?;
    Ok(())
}

pub fn feature_installed(key: &str) -> Result<bool> {
    let doc = leptosx_toml()?;
    Ok(doc["features"][key].as_bool().unwrap_or(false))
}

/// Set the matching key under `[features]` in leptosx.toml.
pub fn set_feature_flag(feature: &str, installed: bool) -> Result<()> {
    let Some(key) = toml_key(feature) else {
        return Ok(());
    };
    let mut doc = leptosx_toml()?;
    doc["features"][key] = value(installed);
    write_leptosx_toml(&doc)
}

/// Add or update a dependency in the project's Cargo.toml. `spec` is a raw
/// TOML value, e.g. `"0.6"` or `{ version = "0.12", features = ["json"] }`.
pub fn add_cargo_dep(dep: &str, spec: &str) -> Result<()> {
    let mut doc = cargo_toml()?;
    doc["dependencies"][dep] = toml_edit::Item::Value(spec.parse::<toml_edit::Value>()?);
    write_cargo_toml(&doc)
}

/// Remove dependencies from the project's Cargo.toml (ignores missing ones).
pub fn remove_cargo_deps(deps: &[&str]) -> Result<()> {
    let mut doc = cargo_toml()?;
    if let Some(table) = doc["dependencies"].as_table_like_mut() {
        for dep in deps {
            table.remove(dep);
        }
    }
    write_cargo_toml(&doc)
}

/// Append `line` to `path` if it is not already present (used for module decls).
pub fn ensure_mod_line(path: &str, line: &str) -> Result<()> {
    let content = fs::read_to_string(path).unwrap_or_default();
    if !content.contains(line) {
        fs::write(path, format!("{content}{line}\n"))?;
    }
    Ok(())
}

/// Remove a module declaration line from `path`.
pub fn drop_mod_line(path: &str, line: &str) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let kept: Vec<&str> = content
        .lines()
        .filter(|l| l.trim() != line)
        .collect();
    if kept.len() != content.lines().count() {
        fs::write(path, format!("{}\n", kept.join("\n")))?;
    }
    Ok(())
}