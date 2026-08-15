//! `cargo-leptosx` — scaffold Leptos frontend projects and manage their features.

mod cli;
mod config;
mod generators;
mod wizard;

use cli::{Cli, Command};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::New {
            name,
            yes,
            rendering,
            styling,
            router,
            ui,
            icons,
            template,
        } => {
            let cfg = config::load();
            generators::new::run(
                &name,
                &generators::new::NewOptions {
                    yes,
                    // CLI flags win; otherwise fall back to ~/.config/leptosx
                    rendering: rendering.or(cfg.rendering),
                    styling: styling.or(cfg.styling),
                    router: router.or(cfg.router),
                    component_lib: ui.or(cfg.ui),
                    icons: icons.or(cfg.icons),
                    template: template.or(cfg.template),
                },
            )
        }
        Command::Add { feature, force, codegen } => generators::add::run(&feature, force, codegen),
        Command::Remove { feature } => generators::remove::run(&feature),
        Command::List => generators::list::run(),
        Command::Doctor => generators::doctor::run(),
    }
}
