use crate::generators::{self, set_feature_flag};
use anyhow::{bail, Result};
use std::fs;

pub fn run(feature: &str) -> Result<()> {
    generators::ensure_project()?;
    generators::validate_feature(feature)?;

    let key = generators::toml_key(feature).unwrap();
    if !generators::feature_installed(key)? {
        bail!("`{feature}` is not installed");
    }

    match feature {
        "tailwind" => remove_tailwind()?,
        "router" => remove_router()?,
        "lucide" => remove_lucide()?,
        "ui" => remove_ui()?,
        "api" => remove_api()?,
        "file-router" => remove_file_router()?,
        _ => unreachable!(),
    }

    set_feature_flag(feature, false)?;
    println!("✔ removed `{}`", feature);
    Ok(())
}

fn remove_tailwind() -> Result<()> {
    let _ = fs::remove_file("tailwind.config.js");
    fs::write(
        "style/main.css",
        "/* app styles */\nbody { font-family: system-ui, sans-serif; margin: 0; padding: 2rem; }\n",
    )?;
    Ok(())
}

fn remove_router() -> Result<()> {
    generators::remove_cargo_deps(&["leptos_router"])?;
    println!(
        "  → removed the dependency; if src/app.rs uses `<Router>`, edit it to drop the router"
    );
    Ok(())
}

fn remove_lucide() -> Result<()> {
    generators::remove_cargo_deps(&["leptos_icons"])?;
    let _ = fs::remove_file("src/components/icons.rs");
    generators::drop_mod_line("src/components/mod.rs", "pub mod icons;")?;
    Ok(())
}

fn remove_ui() -> Result<()> {
    if fs::metadata("src/components/ui").is_ok() {
        let heavy = fs::read_dir("src/components/ui")?.count() > 1;
        if heavy {
            println!("  → leaving src/components/ui/ alone (it contains your code)");
        } else {
            fs::remove_dir_all("src/components/ui")?;
        }
    }
    generators::drop_mod_line("src/components/mod.rs", "pub mod ui;")?;
    Ok(())
}

fn remove_api() -> Result<()> {
    generators::remove_cargo_deps(&["reqwest", "serde"])?;
    let _ = fs::remove_dir_all("src/api");
    generators::drop_mod_line("src/lib.rs", "pub mod api;")?;
    Ok(())
}

fn remove_file_router() -> Result<()> {
    let _ = fs::remove_file("build.rs");
    let _ = fs::remove_file("src/routes.rs");
    generators::drop_mod_line("src/lib.rs", "pub mod routes;")?;

    let mut doc = generators::leptosx_toml()?;
    doc["features"]["file_router_codegen"] = toml_edit::value(false);
    generators::write_leptosx_toml(&doc)?;
    println!("  → removed build.rs; edit src/app.rs if it references the generated routes");
    Ok(())
}