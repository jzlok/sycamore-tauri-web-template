# Tauri + Sycamore Web and App Template
This template should help get you started using Rust to develop standalone ([Tauri](https://v2.tauri.app/)) apps and web applications, both using [Sycamore](https://sycamore.dev/) for UI.

Note that Tauri doesn't insist on using Sycamore for its UI, nor even Rust for your frontend but that focus is the point of this template, which is the equivalent of a Rust, Tauri and Sycamore quickstart (although I am adding helper components over time).  It is a simple 'Hello World' and you will see where to put html into your view component in [src/app.rs](src/app.rs), demonstrating the most basic Sycamore programming right off the bat.

## Prerequisites
- [Rust](https://rustup.rs/) with the `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [`trunk`](https://trunkrs.dev/): `cargo install trunk`
- [`tauri-cli`](https://tauri.app/): `cargo install tauri-cli`
- Optionally, [SCons](https://scons.org/) if you'd rather run `scons <target>` than the raw commands below: `pip install scons`

## Building and running
The repo is a two-crate workspace: the root crate is the Sycamore frontend (compiled to WASM via `trunk`), and `src-tauri/` is the Tauri desktop shell. You can drive both through the [SConstruct](SConstruct), or call `trunk`/`cargo tauri` directly — they do the same thing.

| What you want | SCons | Equivalent direct command |
|---|---|---|
| Frontend only, hot reload, in a browser | `scons web` | `trunk serve` |
| Full desktop app, dev mode | `scons app` | `cargo tauri dev` |
| Production frontend build (outputs to `dist/`) | `scons build-web` | `trunk build --release` |
| Production desktop app/installer | `scons build-app` | `cargo tauri build` |

`scons web` and `trunk serve` only run the Sycamore UI in a browser at `http://localhost:1420` — there's no Tauri window and no `invoke()`-backed commands, so anything that calls into the Rust backend (like the `greet` command) won't work there. Use `scons app` / `cargo tauri dev` to run the real desktop app, which starts `trunk serve` for you under the hood (see `beforeDevCommand` in `src-tauri/tauri.conf.json`).

`scons build-app` / `cargo tauri build` requires platform-specific bundling tools (e.g. WiX or NSIS on Windows) to produce an installer; see the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/) if it fails.

You can also fall back to plain `cargo` for compilation checks without invoking trunk or tauri-cli:

```sh
cargo build -p sycamore-tauri-web-template-ui --target wasm32-unknown-unknown  # frontend
cargo build -p sycamore-tauri-web-template                                    # tauri backend
```

## Docking window layout
As an example of how to use this template comes with a basic docking window UI.

The UI is a menu bar over a dock of tabbed, resizable panels, carried over from the Foundry editor in ferrite0. Drag a tab onto another panel's tab strip or centre to merge it as a tab, or onto an edge to split that panel; drag the dividers to resize; **View → Reset Layout** restores the starting arrangement.

These live in their own crates under `crates/`, so you can keep or drop each one:

| Crate | What it provides | Styles |
|---|---|---|
| [`panel`](crates/panel) | The `Panel` trait: a tab title and content | `crates/panel/panel.css` |
| [`dock`](crates/dock) | `Dock`, `DockLayout` and `DockTree` (depends on `panel`) | `crates/dock/dock.css` |
| [`menu`](crates/menu) | `MenuBar`, `Menu` and `MenuItem` | `crates/menu/menu.css` |
| [`dialog`](crates/dialog) | `Dialog`: a basic modal with an OK button (used for **Help → About**) | `crates/dialog/dialog.css` |

The app's own panels are in [src/panels.rs](src/panels.rs): add a `PanelId` variant, give it a title and content in its `Panel` impl, then place it in `default_layout`. The menus are built in [src/app.rs](src/app.rs). Panel content is rebuilt whenever the layout changes, so keep any long-lived state in context (signals) rather than inside a panel.

To drop a crate, remove it from `[dependencies]` in the root `Cargo.toml`, remove its `<link data-trunk rel="css">` from [index.html](index.html) and its uses in `src/`, and delete its folder. The crates' stylesheets take their colours from the custom properties defined at the top of [styles.css](styles.css).

## Visual Studio Code
There is a .vscode folder in this repository containing an extensions.json to assist in quicker setup for that environment.
