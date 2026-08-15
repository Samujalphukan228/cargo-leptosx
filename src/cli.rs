use clap::{Parser, Subcommand};
use crate::wizard::{Icons, Rendering, Styling};

#[derive(Parser)]
#[command(name = "cargo-leptosx", bin_name = "cargo")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Scaffold a new Leptos frontend project
    New {
        /// Project directory / name
        name: String,

        /// Skip the wizard and use defaults (overridable with the flags below)
        #[arg(short, long)]
        yes: bool,

        /// Rendering mode: csr or ssr-hydration
        #[arg(long, value_enum)]
        rendering: Option<Rendering>,

        /// Styling: tailwind, css, or none
        #[arg(long, value_enum)]
        styling: Option<Styling>,

        /// Add leptos_router. Accepts --router or --router=false
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        router: Option<bool>,

        /// Add a component library skeleton. Accepts --ui or --ui=false
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        ui: Option<bool>,

        /// Icons: none or lucide
        #[arg(long, value_enum)]
        icons: Option<Icons>,

        /// Starter template: minimal (one page) or full (about/contact + UI kit)
        #[arg(long)]
        template: Option<String>,
    },
    /// Add a feature (tailwind, router, lucide, ui, api, file-router) to an existing project
    Add {
        feature: String,

        /// Re-run even if the feature is already installed
        #[arg(long)]
        force: bool,

        /// With `file-router`: write routes to src/routes.rs (checked in)
        /// instead of OUT_DIR at build time
        #[arg(long)]
        codegen: bool,
    },
    /// Remove a feature from an existing project
    Remove {
        feature: String,
    },
    /// Show which features are installed in the current project
    List,
    /// Check the current project for inconsistencies (leptosx.toml vs Cargo.toml)
    Doctor,
}

impl Cli {
    /// Cargo invokes subcommands as `cargo-leptosx leptosx <args>`, so strip
    /// the leading "leptosx" so `cargo leptosx new` and direct invocation both work.
    pub fn parse() -> Self {
        let mut args: Vec<String> = std::env::args().collect();
        if args.len() > 1 && args[1] == "leptosx" {
            args.remove(1);
        }
        <Cli as clap::Parser>::parse_from(args)
    }
}
