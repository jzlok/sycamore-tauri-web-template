//! The Sycamore frontend's root component.
//!
//! This crate is compiled to WASM and runs inside the Tauri webview (or a
//! plain browser via `trunk serve`/`scons web` — see the README).

use sycamore::prelude::*;

#[component]
pub fn App() -> View {
    view! {
        main(class="container") {
            h1 {
                "Hello, world!"
            }
        }
    }
}
