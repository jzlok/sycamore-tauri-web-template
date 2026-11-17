//! This app's panels, and where they start in the dock.
//!
//! To add a panel: add a [`PanelId`] variant, give it a title and content in the
//! [`Panel`] impl, and place it in [`default_layout`].

use dock::{DockTree, Panel};
use sycamore::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PanelId {
    Panel1,
    Panel2,
    Panel3,
    Panel4,
}

impl Panel for PanelId {
    fn title(self) -> &'static str {
        match self {
            PanelId::Panel1 => "Panel 1",
            PanelId::Panel2 => "Panel 2",
            PanelId::Panel3 => "Panel 3",
            PanelId::Panel4 => "Panel 4",
        }
    }

    fn content(self) -> View {
        match self {
            PanelId::Panel2 => view! {
                div(class="panel hello-panel") {
                    h1 { "Hello, world!" }
                    p(class="panel-hint") { "Drag a tab onto another panel to dock it there." }
                }
            },
            other => view! {
                div(class="panel") {
                    p(class="panel-hint") { (other.title()) }
                }
            },
        }
    }
}

/// The starting layout: Panel 1 | (Panel 2 over Panel 3) | Panel 4.
pub fn default_layout() -> DockTree<PanelId> {
    DockTree::row([
        (1.0, DockTree::leaf([PanelId::Panel1])),
        (
            3.5,
            DockTree::column([
                (3.0, DockTree::leaf([PanelId::Panel2])),
                (1.0, DockTree::leaf([PanelId::Panel3])),
            ]),
        ),
        (1.3, DockTree::leaf([PanelId::Panel4])),
    ])
}
