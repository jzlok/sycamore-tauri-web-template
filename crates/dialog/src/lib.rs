//! A basic modal dialog: a title, some content and an OK button that closes it.
//! While it is open, a backdrop covers (and blocks) the rest of the page.
//!
//! ```ignore
//! let about_open = create_signal(false);
//! view! {
//!     Dialog(open=about_open, title="About") {
//!         p { "My App, version 1.0" }
//!     }
//! }
//! // Somewhere else: about_open.set(true);
//! ```
//!
//! Styles are in `dialog.css`, next to this crate's `Cargo.toml`.

use sycamore::prelude::*;

#[component(inline_props)]
pub fn Dialog(open: Signal<bool>, title: &'static str, children: Children) -> View {
    let content = children.call();

    view! {
        div(class="dialog-backdrop", hidden=move || !open.get()) {
            div(class="dialog", role="dialog", aria-modal="true", aria-label=title) {
                div(class="dialog-title") { (title) }
                div(class="dialog-body") { (content) }
                div(class="dialog-buttons") {
                    button(on:click=move |_| open.set(false)) { "OK" }
                }
            }
        }
    }
}
