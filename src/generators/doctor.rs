//! `doctor` — check a project's leptosx.toml against Cargo.toml and the
//! generated files, reporting inconsistencies.

use crate::generators::{self, ALL_FEATURES};
use anyhow::Result;
use std::fs;

pub fn run() -> Result<()> {
    generators::ensure_project()?;
    let mut problems = 0usize;

    let doc = generators::leptosx_toml()?;
    let cargo = match fs::metadata("Cargo.toml") {
        Ok(_) => generators::cargo_toml()?,
        Err(_) => {
            println!("  ✗ Cargo.toml missing");
            problems += 1;
            toml_edit::Document::new()
        }
    };

    for feature in ALL_FEATURES {
        let key = generators::toml_key(feature).unwrap();
        let installed = generators::feature_installed(key).unwrap_or(false);
        let dep = match *feature {
            "router" => Some("leptos_router"),
            "lucide" => Some("leptos_icons"),
            "api" => Some("reqwest"),
            _ => None,
        };

        if installed {
            if let Some(dep) = dep {
                if !has_dep(&cargo, dep) {
                    println!("  ✗ `{feature}` is marked installed in leptosx.toml but `{dep}` is missing from Cargo.toml");
                    problems += 1;
                }
            }
        } else if let Some(dep) = dep {
            if has_dep(&cargo, dep) {
                println!("  ~ `{dep}` is in Cargo.toml but `{feature}` is not marked installed");
            }
        }
    }

    if doc["project"]["rendering"].as_str() == Some("csr") && fs::metadata("index.html").is_err() {
        println!("  ✗ CSR project missing index.html (needed by trunk)");
        problems += 1;
    }

    let file_router = generators::feature_installed("file_router").unwrap_or(false);
    if file_router && fs::metadata("build.rs").is_err() {
        println!("  ✗ file-router is enabled but build.rs is missing — run `cargo leptosx add file-router --force`");
        problems += 1;
    }
    if file_router
        && generators::feature_installed("file_router_codegen").unwrap_or(false)
        && fs::metadata("src/routes.rs").is_err()
    {
        println!("  ✗ file-router codegen is enabled but src/routes.rs is missing");
        problems += 1;
    }

    if problems == 0 {
        println!("✔ everything looks good");
    } else {
        println!("  {problems} issue(s) found");
    }
    Ok(())
}

fn has_dep(cargo: &toml_edit::Document, dep: &str) -> bool {
    cargo
        .get("dependencies")
        .and_then(|d| d.as_table_like())
        .map(|t| t.get(dep).is_some())
        .unwrap_or(false)
}