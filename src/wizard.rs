use dialoguer::{theme::ColorfulTheme, Select, Confirm};

pub struct Answers {
    pub rendering: Rendering,
    pub styling: Styling,
    pub router: bool,
    pub component_lib: bool,
    pub icons: Icons,
}

#[derive(Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum Rendering { Csr, SsrHydration }
#[derive(Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum Styling { Tailwind, Css, None }
#[derive(Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum Icons { None, Lucide }

pub fn run() -> anyhow::Result<Answers> {
    let theme = ColorfulTheme::default();

    let rendering = Select::with_theme(&theme)
        .with_prompt("Rendering")
        .items(&["CSR", "SSR + Hydration"])
        .default(0)
        .interact()?;

    let styling = Select::with_theme(&theme)
        .with_prompt("Styling")
        .items(&["Tailwind CSS", "CSS", "None"])
        .default(0)
        .interact()?;

    let router = Confirm::with_theme(&theme)
        .with_prompt("Add Leptos Router?")
        .default(true)
        .interact()?;

    let component_lib = Confirm::with_theme(&theme)
        .with_prompt("Add a component library?")
        .default(false)
        .interact()?;

    let icons = Select::with_theme(&theme)
        .with_prompt("Icons")
        .items(&["None", "Lucide"])
        .default(0)
        .interact()?;

    Ok(Answers {
        rendering: if rendering == 0 { Rendering::Csr } else { Rendering::SsrHydration },
        styling: match styling {
            0 => Styling::Tailwind,
            1 => Styling::Css,
            _ => Styling::None,
        },
        router,
        component_lib,
        icons: if icons == 0 { Icons::None } else { Icons::Lucide },
    })
}
