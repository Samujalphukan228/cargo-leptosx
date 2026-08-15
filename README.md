# cargo-leptosx

A `cargo` subcommand that scaffolds [Leptos](https://leptos.dev) frontend
projects and manages their features. One command from zero to a running app —
with **file-based routing** (Next.js-style) built in.

**Frontend only by design.** No backend, no database. Pair the generated
frontend with any backend you choose.

## Install

```sh
cargo install cargo-leptosx
```

> You can also run it straight from a checkout: `cargo install --path .`

## Quick start

```sh
cargo leptosx new my-app
cd my-app

# CSR app (trunk):
cargo install trunk        # once
trunk serve --open

# Or SSR + hydration (cargo-leptos + axum):
cargo install cargo-leptos # once
cargo leptos watch
```

Skip the interactive wizard with flags:

```sh
cargo leptosx new my-app --yes --rendering csr --styling tailwind --template full
```

### Options

| Flag | Values | Default |
|---|---|---|
| `--rendering` | `csr`, `ssr-hydration` | `csr` |
| `--styling` | `tailwind`, `css`, `none` | `tailwind` |
| `--router[=bool]` | — | `true` (installs file-based routing) |
| `--ui[=bool]` | — | `false` |
| `--icons` | `none`, `lucide` | `lucide` |
| `--template` | `minimal`, `full` | `minimal` |
| `--yes` | — | skip the wizard |

Defaults for flagless runs can come from `~/.config/leptosx/config.toml` (see
[Defaults](#defaults)).

## File-based routing

Choosing `--router` (the default) installs file-based routing automatically:
just create a file or folder in `src/pages/` and it becomes a route. No manual
routing.

```
src/pages/index.rs            -> "/"
src/pages/about.rs            -> "/about"
src/pages/blog/index.rs       -> "/blog"
src/pages/blog/[slug].rs      -> "/blog/:slug"
src/pages/blog/+layout.rs     -> wraps everything under /blog
src/pages/settings/[...rest]  -> "/settings/*rest"
```

Conventions:

- every file in `src/pages/` exports a `Page` component
- every `+layout.rs` exports a `Layout` component rendering an `<Outlet/>`
- `[slug]` and `[...rest]` become dynamic/rest params
- a catch-all `*path` route shows a built-in 404
- duplicate routes fail the build with a clear message

### How it works

The tool drops a `build.rs` into your project. It scans `src/pages/` on every
build and emits a `pub fn routes()` — a `<Routes>` block — that `src/app.rs`
`include!`s. By default routes live in `OUT_DIR`; prefer them checked in?

```sh
cargo leptosx add file-router --codegen   # writes src/routes.rs
```

## Features

Manage features on an existing project. State is tracked in `leptosx.toml`
(the single source of truth), and `add`/`remove` keep it and `Cargo.toml` in
sync.

| Command | What it does |
|---|---|
| `cargo leptosx add tailwind` | scaffolds `tailwind.config.js` + `style/main.css` |
| `cargo leptosx add router` | adds `leptos_router` |
| `cargo leptosx add lucide` | adds `leptos_icons` + a re-export module |
| `cargo leptosx add ui` | creates `src/components/ui/` primitives |
| `cargo leptosx add api` | typed `reqwest` fetch wrapper in `src/api/` |
| `cargo leptosx add file-router` | file-based routing (see above) |
| `cargo leptosx remove <feature>` | undoes `add` |
| `cargo leptosx list` | shows installed features |
| `cargo leptosx doctor` | checks the project is consistent |

`add` refuses to re-apply an installed feature unless `--force` is given.
`remove file-router` also cleans up `build.rs`, `src/routes.rs`, and any
`pub mod routes;` declaration.

## Defaults

Save preferred defaults to `~/.config/leptosx/config.toml`. Flags always win
over these values.

```toml
[new]
rendering = "csr"        # csr | ssr-hydration
styling = "tailwind"     # tailwind | css | none
router = true
ui = false
icons = "lucide"         # none | lucide
template = "minimal"     # minimal | full
```

## What gets generated

- **CSR** (`--rendering csr`): a trunk project — `Cargo.toml` (cdylib lib),
  `src/main.rs` mounting `<App/>`, `index.html`, `style/`, and
  `src/{app,pages,components}/`.
- **SSR + Hydration** (`--rendering ssr-hydration`): a cargo-leptos + axum
  project with `src/main.rs` (server), `src/lib.rs` (`hydrate`), error
  template, and static-file serving.

Both variants produce projects that compile out of the box — verified in CI
against the `wasm32-unknown-unknown` target.