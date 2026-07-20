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

## Visual Studio Code
There is a .vscode folder in this repository containing an extensions.json to assist in quicker setup for that environment.
