//! The Sycamore frontend's root component.
//!
//! This crate is compiled to WASM and runs inside the Tauri webview (or a
//! plain browser via `trunk serve`/`scons web` — see the README).
//!
//! The window is a menu bar (the `menu` crate) over a dock of tabbed, resizable
//! panels (the `dock` crate), plus an About box (the `dialog` crate). This
//! app's panels are in [`crate::panels`].

use dialog::Dialog;
use dock::{Dock, DockLayout};
use menu::{Menu, MenuBar, MenuItem};
use sycamore::prelude::*;

use crate::panels::default_layout;

#[component]
pub fn App() -> View {
    let layout = DockLayout::new(default_layout());
    let about_open = create_signal(false);

    let menus = vec![
        Menu::new("File", [MenuItem::disabled("New"), MenuItem::disabled("Open"), MenuItem::disabled("Save")]),
        Menu::new("Edit", [MenuItem::disabled("Undo"), MenuItem::disabled("Redo"), MenuItem::disabled("Preferences")]),
        Menu::new("View", [MenuItem::new("Reset Layout", move || layout.reset())]),
        Menu::new("Help", [MenuItem::new("About", move || about_open.set(true))]),
    ];

    view! {
        div(class="app") {
            MenuBar(menus=menus)
            Dock(layout=layout)
            Dialog(open=about_open, title="About") {
                p { "Rust, Tauri and Sycamore Web/App Template" }
                p(class="panel-hint") { (format!("Version {}", env!("CARGO_PKG_VERSION"))) }
            }
        }
    }
}
