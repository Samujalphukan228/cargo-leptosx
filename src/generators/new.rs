use crate::wizard::{self, Answers, Icons, Rendering, Styling};
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Options coming from the CLI. When any field is set (or `yes` is given),
/// the wizard is skipped.
pub struct NewOptions {
    pub yes: bool,
    pub rendering: Option<Rendering>,
    pub styling: Option<Styling>,
    pub router: Option<bool>,
    pub component_lib: Option<bool>,
    pub icons: Option<Icons>,
    pub template: Option<String>,
}

impl NewOptions {
    fn interactive(&self) -> bool {
        !self.yes
            && self.rendering.is_none()
            && self.styling.is_none()
            && self.router.is_none()
            && self.component_lib.is_none()
            && self.icons.is_none()
            && self.template.is_none()
    }

    fn resolve(&self) -> Answers {
        Answers {
            rendering: self.rendering.unwrap_or(Rendering::Csr),
            styling: self.styling.unwrap_or(Styling::Tailwind),
            router: self.router.unwrap_or(true),
            component_lib: self.component_lib.unwrap_or(false),
            icons: self.icons.unwrap_or(Icons::Lucide),
        }
    }
}

pub fn run(name: &str, opts: &NewOptions) -> Result<()> {
    let answers = if opts.interactive() {
        wizard::run()?
    } else {
        opts.resolve()
    };
    let template = opts.template.as_deref().unwrap_or("minimal");
    generate(name, &answers, template)
}

fn generate(name: &str, answers: &Answers, template: &str) -> Result<()> {
    let root = Path::new(name);
    if root.exists() {
        anyhow::bail!("directory `{}` already exists", name);
    }
    let pkg_name = root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| name.to_string());
    let result = (|| -> Result<()> {
        for dir in ["src/components/ui", "src/pages", "public", "style"] {
            fs::create_dir_all(root.join(dir))?;
        }
        match answers.rendering {
            Rendering::Csr => generate_csr(root, pkg_name.as_str(), answers, template)?,
            Rendering::SsrHydration => generate_ssr(root, pkg_name.as_str(), answers, template)?,
        }
        write_leptosx_toml(root, pkg_name.as_str(), answers)?;
        // Router selected -> auto-install file-based routing so folders in
        // src/pages/ become routes (Next.js-style).
        if answers.router {
            crate::generators::pages_scan::install_in(root, false)?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(root);
    }
    result?;

    println!("✔ created `{}`", pkg_name);
    println!();
    match answers.rendering {
        Rendering::Csr => {
            println!("  next:");
            println!("    cd {pkg_name}");
            println!("    cargo install trunk          # once");
            println!("    trunk serve --open");
        }
        Rendering::SsrHydration => {
            println!("  next:");
            println!("    cd {pkg_name}");
            println!("    cargo install cargo-leptos    # once");
            println!("    cargo leptos watch");
        }
    }
    if answers.router {
        println!("  tip: add pages as files/folders under src/pages/ — each becomes a route");
    }
    Ok(())
}

/// Fill `{name}` / `{crate_name}` placeholders (distinct from the `{}` used
/// in the Rust/toml template bodies).
fn fill(template: &str, name: &str, crate_name: &str) -> String {
    template
        .replace("{name}", name)
        .replace("{crate_name}", crate_name)
}

fn generate_csr(root: &Path, name: &str, answers: &Answers, template: &str) -> Result<()> {
    let crate_name = name.replace('-', "_");

    let mut deps = vec![
        "leptos = { version = \"0.6\", features = [\"csr\"] }".to_string(),
        "console_error_panic_hook = \"0.1\"".to_string(),
    ];
    if answers.router {
        deps.push("leptos_router = \"0.6\"".to_string());
    }
    if matches!(answers.icons, Icons::Lucide) {
        deps.push("leptos_icons = \"0.3\"".to_string());
    }

    let cargo_toml = fill(CSR_CARGO_TOML, name, &crate_name).replace("{deps}", &deps.join("\n"));
    fs::write(root.join("Cargo.toml"), cargo_toml)?;

    fs::write(root.join("src/lib.rs"), CSR_LIB_RS)?;
    fs::write(
        root.join("src/main.rs"),
        fill(CSR_MAIN_RS, name, &crate_name),
    )?;

    // When a router is selected, install_in() rewrites app.rs with the
    // generated file-router, so this is only the no-router fallback.
    fs::write(root.join("src/app.rs"), CSR_APP_PLAIN_RS)?;

    let full = template == "full";
    write_pages(
        root,
        CSR_INDEX_PAGE_RS,
        CSR_ABOUT_PAGE_RS,
        CSR_CONTACT_PAGE_RS,
        full,
    )?;
    write_components(root, answers, full)?;

    fs::write(root.join("index.html"), fill(CSR_INDEX_HTML, name, ""))?;
    write_styling(root, name, answers, true)?;

    Ok(())
}

fn generate_ssr(root: &Path, name: &str, answers: &Answers, template: &str) -> Result<()> {
    let crate_name = name.replace('-', "_");

    fs::write(
        root.join("Cargo.toml"),
        fill(SSR_CARGO_TOML, name, &crate_name),
    )?;

    fs::write(root.join("src/lib.rs"), fill(SSR_LIB_RS, name, &crate_name))?;
    fs::write(root.join("src/main.rs"), fill(SSR_MAIN_RS, name, &crate_name))?;
    let full = template == "full";
    fs::write(root.join("src/app.rs"), fill(SSR_APP_RS, name, &crate_name))?;
    fs::write(root.join("src/error_template.rs"), SSR_ERROR_TEMPLATE_RS)?;
    fs::write(root.join("src/fileserv.rs"), SSR_FILESERV_RS)?;

    write_pages(
        root,
        SSR_INDEX_PAGE_RS,
        SSR_ABOUT_PAGE_RS,
        SSR_CONTACT_PAGE_RS,
        full,
    )?;
    write_components(root, answers, full)?;
    write_styling(root, name, answers, false)?;

    Ok(())
}

fn write_pages(root: &Path, index: &str, about: &str, contact: &str, full: bool) -> Result<()> {
    fs::write(
        root.join("src/pages/mod.rs"),
        if full {
            "pub mod about;\npub mod contact;\npub mod index;\n"
        } else {
            "pub mod index;\n"
        },
    )?;
    fs::write(root.join("src/pages/index.rs"), index)?;
    if full {
        fs::write(root.join("src/pages/about.rs"), about)?;
        fs::write(root.join("src/pages/contact.rs"), contact)?;
    }
    Ok(())
}

fn write_components(root: &Path, answers: &Answers, full: bool) -> Result<()> {
    if full || answers.component_lib {
        fs::write(root.join("src/components/mod.rs"), "pub mod ui;\n")?;
        fs::write(
            root.join("src/components/ui/mod.rs"),
            if full { CSR_UI_LIB_RS } else { CSR_BUTTON_RS },
        )?;
    } else {
        fs::write(root.join("src/components/mod.rs"), "// components go here\n")?;
    }
    Ok(())
}

fn write_styling(root: &Path, name: &str, answers: &Answers, is_csr: bool) -> Result<()> {
    match answers.styling {
        Styling::Tailwind => {
            fs::write(root.join("style/main.css"), TAILWIND_CSS)?;
            fs::write(root.join("tailwind.config.js"), TAILWIND_CONFIG_JS)?;
        }
        Styling::Css => {
            fs::write(root.join("style/main.css"), CSS_MAIN)?;
            if is_csr {
                fs::write(root.join("index.html"), fill(CSR_INDEX_HTML_CSS, name, ""))?;
            }
        }
        Styling::None => {
            if !is_csr {
                fs::write(root.join("style/main.css"), "/* styles */\n")?;
            }
        }
    }
    Ok(())
}

fn write_leptosx_toml(root: &Path, name: &str, answers: &Answers) -> Result<()> {
    let toml = format!(
        "[project]\nname = \"{name}\"\nrendering = \"{rendering}\"\n\n[features]\ntailwind = {tailwind}\nrouter = {router}\ncomponent_lib = {component_lib}\nlucide = {lucide}\nfile_router = {file_router}\nfile_router_codegen = false\napi = false\n",
        name = name,
        rendering = match answers.rendering { Rendering::Csr => "csr", Rendering::SsrHydration => "ssr-hydration" },
        tailwind = matches!(answers.styling, Styling::Tailwind),
        router = answers.router || matches!(answers.rendering, Rendering::SsrHydration),
        component_lib = answers.component_lib,
        lucide = matches!(answers.icons, Icons::Lucide),
        file_router = answers.router,
    );
    fs::write(root.join("leptosx.toml"), toml)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// CSR (trunk) templates
// ---------------------------------------------------------------------------

const CSR_CARGO_TOML: &str = r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
{deps}"#;

const CSR_LIB_RS: &str = "pub mod app;\npub mod components;\npub mod pages;\n";

const CSR_MAIN_RS: &str = r#"use leptos::*;
use {crate_name}::app::App;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
"#;

const CSR_APP_PLAIN_RS: &str = r#"use crate::pages;
use leptos::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <pages::index::Page/>
    }
}
"#;

const CSR_INDEX_PAGE_RS: &str = r#"use leptos::*;

#[component]
pub fn Page() -> impl IntoView {
    view! {
        <h1>"Welcome to leptosx"</h1>
        <p>"Edit src/pages/index.rs and the dev server will hot-reload."</p>
    }
}
"#;

const CSR_BUTTON_RS: &str = r#"use leptos::*;

/// A tiny example component. Put your real UI primitives here.
#[component]
pub fn Button(children: Children) -> impl IntoView {
    view! {
        <button class="btn">{children()}</button>
    }
}
"#;

const CSR_UI_LIB_RS: &str = r#"// A small component library. Add your primitives here, e.g.:
//   pub mod button;
//   pub mod card;
use leptos::*;

#[component]
pub fn Button(children: Children) -> impl IntoView {
    view! {
        <button class="btn">{children()}</button>
    }
}

#[component]
pub fn Card(children: Children) -> impl IntoView {
    view! {
        <div class="card">{children()}</div>
    }
}
"#;

const CSR_ABOUT_PAGE_RS: &str = r#"use leptos::*;

#[component]
pub fn Page() -> impl IntoView {
    view! {
        <h1>"About"</h1>
        <p>"This project was scaffolded with cargo-leptosx."</p>
    }
}
"#;

const CSR_CONTACT_PAGE_RS: &str = r#"use leptos::*;

#[component]
pub fn Page() -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    view! {
        <h1>"Contact"</h1>
        <form>
            <label>"Name"</label>
            <input
                prop:value=name
                on:input=move |ev| set_name.set(event_target_value(&ev))
            />
            <p>"Hello, " {name} "!"</p>
        </form>
    }
}
"#;

const CSR_INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1"/>
    <link data-trunk rel="rust"/>
    <title>{name}</title>
  </head>
  <body></body>
</html>
"#;

const CSR_INDEX_HTML_CSS: &str = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1"/>
    <link data-trunk rel="rust"/>
    <link rel="stylesheet" href="style/main.css"/>
    <title>{name}</title>
  </head>
  <body></body>
</html>
"#;

const CSS_MAIN: &str = r#"/* app styles */
body {
  font-family: system-ui, sans-serif;
  margin: 0;
  padding: 2rem;
}
h1 {
  color: #0f766e;
}
"#;

const TAILWIND_CSS: &str = r#"@tailwind base;
@tailwind components;
@tailwind utilities;
"#;

const TAILWIND_CONFIG_JS: &str = r#"/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.rs", "./index.html"],
  theme: {
    extend: {},
  },
  plugins: [],
};
"#;

// ---------------------------------------------------------------------------
// SSR + Hydration (cargo-leptos + axum) templates, based on leptos-rs/start-axum
// ---------------------------------------------------------------------------

const SSR_CARGO_TOML: &str = r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
axum = { version = "0.7", optional = true }
console_error_panic_hook = "0.1"
http = "1"
leptos = { version = "0.6" }
leptos_axum = { version = "0.6", optional = true }
leptos_meta = { version = "0.6" }
leptos_router = { version = "0.6" }
thiserror = "1"
tokio = { version = "1", features = ["rt-multi-thread"], optional = true }
tower = { version = "0.4", optional = true, features = ["util"] }
tower-http = { version = "0.5", features = ["fs"], optional = true }
tracing = { version = "0.1", optional = true }
wasm-bindgen = "=0.2.95"

[features]
hydrate = ["leptos/hydrate", "leptos_meta/hydrate", "leptos_router/hydrate"]
ssr = [
    "dep:axum",
    "dep:tokio",
    "dep:tower",
    "dep:tower-http",
    "dep:leptos_axum",
    "leptos/ssr",
    "leptos_meta/ssr",
    "leptos_router/ssr",
    "dep:tracing",
]

[profile.wasm-release]
inherits = "release"
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"

[package.metadata.leptos]
output-name = "{name}"
site-root = "target/site"
site-pkg-dir = "pkg"
style-file = "style/main.css"
assets-dir = "public"
site-addr = "127.0.0.1:3000"
reload-port = 3001
env = "DEV"
bin-features = ["ssr"]
bin-default-features = false
lib-features = ["hydrate"]
lib-default-features = false
lib-profile-release = "wasm-release"
"#;

const SSR_MAIN_RS: &str = r#"#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use {crate_name}::app::*;
    use {crate_name}::fileserv::file_and_error_handler;

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, App)
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
"#;
const SSR_LIB_RS: &str = r#"pub mod app;
pub mod components;
pub mod error_template;
#[cfg(feature = "ssr")]
pub mod fileserv;
pub mod pages;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
"#;

const SSR_APP_RS: &str = r#"use crate::error_template::{AppError, ErrorTemplate};
use crate::pages;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/{crate_name}.css"/>
        <Title text="Welcome to leptosx"/>

        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=pages::index::Page/>
                </Routes>
            </main>
        </Router>
    }
}
"#;

const SSR_INDEX_PAGE_RS: &str = r#"use leptos::*;

#[component]
pub fn Page() -> impl IntoView {
    view! {
        <h1>"Welcome to leptosx (SSR)"</h1>
        <p>"Server-rendered with hydration. Edit src/pages/index.rs and cargo leptos watch will hot-reload."</p>
    }
}
"#;

const SSR_ABOUT_PAGE_RS: &str = r#"use leptos::*;

#[component]
pub fn Page() -> impl IntoView {
    view! {
        <h1>"About"</h1>
        <p>"This project was scaffolded with cargo-leptosx (SSR + hydration)."</p>
    }
}
"#;

const SSR_CONTACT_PAGE_RS: &str = r#"use leptos::*;

#[component]
pub fn Page() -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    view! {
        <h1>"Contact"</h1>
        <form>
            <label>"Name"</label>
            <input
                prop:value=name
                on:input=move |ev| set_name.set(event_target_value(&ev))
            />
            <p>"Hello, " {name} "!"</p>
        </form>
    }
}
"#;

const SSR_ERROR_TEMPLATE_RS: &str = r#"use http::status::StatusCode;
use leptos::*;
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum AppError {
    #[error("Not Found")]
    NotFound,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

#[component]
pub fn ErrorTemplate(
    #[prop(optional)] outside_errors: Option<Errors>,
    #[prop(optional)] errors: Option<RwSignal<Errors>>,
) -> impl IntoView {
    let errors = match outside_errors {
        Some(e) => create_rw_signal(e),
        None => match errors {
            Some(e) => e,
            None => panic!("No Errors found and we expected errors!"),
        },
    };
    let errors = errors.get_untracked();

    let errors: Vec<AppError> = errors
        .into_iter()
        .filter_map(|(_k, v)| v.downcast_ref::<AppError>().cloned())
        .collect();

    #[cfg(feature = "ssr")]
    {
        use leptos_axum::ResponseOptions;
        let response = use_context::<ResponseOptions>();
        if let Some(response) = response {
            response.set_status(errors[0].status_code());
        }
    }

    view! {
        <h1>{if errors.len() > 1 {"Errors"} else {"Error"}}</h1>
        <For
            each= move || {errors.clone().into_iter().enumerate()}
            key=|(index, _error)| *index
            children=move |error| {
                let error_string = error.1.to_string();
                let error_code = error.1.status_code();
                view! {
                    <h2>{error_code.to_string()}</h2>
                    <p>"Error: " {error_string}</p>
                }
            }
        />
    }
}
"#;

const SSR_FILESERV_RS: &str = r#"use crate::app::App;
use axum::response::Response as AxumResponse;
use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    response::IntoResponse,
};
use leptos::*;
use tower::ServiceExt;
use tower_http::services::ServeDir;

pub async fn file_and_error_handler(
    State(options): State<LeptosOptions>,
    req: Request<Body>,
) -> AxumResponse {
    let root = options.site_root.clone();
    let (parts, body) = req.into_parts();

    let mut static_parts = parts.clone();
    static_parts.headers.clear();
    if let Some(encodings) = parts.headers.get("accept-encoding") {
        static_parts
            .headers
            .insert("accept-encoding", encodings.clone());
    }

    let res = get_static_file(Request::from_parts(static_parts, Body::empty()), &root)
        .await
        .unwrap();

    if res.status() == StatusCode::OK {
        res.into_response()
    } else {
        let handler = leptos_axum::render_app_to_stream(options.to_owned(), App);
        handler(Request::from_parts(parts, body))
            .await
            .into_response()
    }
}

async fn get_static_file(
    request: Request<Body>,
    root: &str,
) -> Result<Response<Body>, (StatusCode, String)> {
    match ServeDir::new(root)
        .precompressed_gzip()
        .precompressed_br()
        .oneshot(request)
        .await
    {
        Ok(res) => Ok(res.into_response()),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error serving files: {err}"),
        )),
    }
}
"#;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::wizard::{Rendering, Styling};
    use std::sync::Mutex;

    // set_current_dir is process-global, so serialize these tests.
    static CWD_LOCK: Mutex<()> = Mutex::new(());

    fn answers() -> Answers {
        Answers {
            rendering: Rendering::Csr,
            styling: Styling::Css,
            router: true,
            component_lib: true,
            icons: Icons::Lucide,
        }
    }

    fn answers_ssr() -> Answers {
        Answers {
            rendering: Rendering::SsrHydration,
            styling: Styling::Css,
            router: true,
            component_lib: false,
            icons: Icons::None,
        }
    }

    fn in_tempdir(body: impl FnOnce()) {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        body();
        std::env::set_current_dir(&prev).unwrap();
    }

    #[test]
    fn generates_full_csr_project() {
        in_tempdir(|| {
            generate("myapp", &answers(), "full").unwrap();
            let root = Path::new("myapp");
            for f in [
                "Cargo.toml",
                "index.html",
                "leptosx.toml",
                "src/main.rs",
                "src/lib.rs",
                "src/app.rs",
                "src/pages/index.rs",
                "src/pages/about.rs",
                "src/pages/contact.rs",
                "src/components/ui/mod.rs",
            ] {
                assert!(root.join(f).exists(), "missing {f}");
            }
            let cargo = fs::read_to_string("myapp/Cargo.toml").unwrap();
            assert!(cargo.contains("leptos_router"), "router dep missing");
            assert!(cargo.contains("leptos_icons"), "icons dep missing");
            assert!(cargo.contains("crate-type"), "missing [lib] crate-type");
        });
    }

    #[test]
    fn generates_ssr_project() {
        in_tempdir(|| {
            generate("ssrapp", &answers_ssr(), "minimal").unwrap();
            for f in [
                "ssrapp/src/main.rs",
                "ssrapp/src/lib.rs",
                "ssrapp/src/error_template.rs",
                "ssrapp/src/fileserv.rs",
            ] {
                assert!(Path::new(f).exists(), "missing {f}");
            }
            let cargo = fs::read_to_string("ssrapp/Cargo.toml").unwrap();
            assert!(cargo.contains("[features]"));
            assert!(cargo.contains("ssr = ["));
            assert!(cargo.contains("hydrate = ["));
        });
    }

    #[test]
    fn refuses_existing_directory() {
        in_tempdir(|| {
            fs::create_dir("myapp").unwrap();
            let result = generate("myapp", &answers(), "minimal");
            assert!(result.is_err());
        });
    }
}
