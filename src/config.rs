use crate::wizard::{Icons, Rendering, Styling};
use std::path::PathBuf;
use toml_edit::Document;

/// User-level defaults, saved to `~/.config/leptosx/config.toml` so the wizard
/// (or flagless invocations) can start from a user's preferred choices.
///
/// ```toml
/// [new]
/// rendering = "csr"            # csr | ssr-hydration
/// styling = "tailwind"         # tailwind | css | none
/// router = true
/// ui = false
/// icons = "lucide"             # none | lucide
/// template = "minimal"         # minimal | full
/// ```
#[derive(Default)]
pub struct Config {
    pub rendering: Option<Rendering>,
    pub styling: Option<Styling>,
    pub router: Option<bool>,
    pub ui: Option<bool>,
    pub icons: Option<Icons>,
    pub template: Option<String>,
}

pub fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".config/leptosx/config.toml")
}

pub fn load() -> Config {
    let mut cfg = Config::default();
    let path = config_path();
    if !path.exists() {
        return cfg;
    }
    let Ok(content) = std::fs::read_to_string(&path) else {
        return cfg;
    };
    let Ok(doc) = content.parse::<Document>() else {
        return cfg;
    };
    cfg.rendering = doc["new"]["rendering"]
        .as_str()
        .and_then(|s| match s {
            "csr" => Some(Rendering::Csr),
            "ssr-hydration" => Some(Rendering::SsrHydration),
            _ => None,
        });
    cfg.styling = doc["new"]["styling"]
        .as_str()
        .and_then(|s| match s {
            "tailwind" => Some(Styling::Tailwind),
            "css" => Some(Styling::Css),
            "none" => Some(Styling::None),
            _ => None,
        });
    cfg.router = doc["new"]["router"].as_bool();
    cfg.ui = doc["new"]["ui"].as_bool();
    cfg.icons = doc["new"]["icons"]
        .as_str()
        .and_then(|s| match s {
            "none" => Some(Icons::None),
            "lucide" => Some(Icons::Lucide),
            _ => None,
        });
    cfg.template = doc["new"]["template"].as_str().map(String::from);
    cfg
}