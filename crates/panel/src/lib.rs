//! A panel is a piece of UI that can be shown in a dock (see the `dock` crate).
//!
//! An app usually implements [`Panel`] on a plain enum with one variant per panel.
//!
//! Styles for panel content (`.panel`, `.panel-hint`) are in `panel.css`, next to
//! this crate's `Cargo.toml`.

use sycamore::prelude::*;

pub trait Panel: Copy + Eq + 'static {
    /// The panel's tab label.
    fn title(self) -> &'static str;

    /// Builds the panel's content. A dock calls this again every time its layout
    /// changes, so panels should read their state from context rather than own it.
    fn content(self) -> View;
}
