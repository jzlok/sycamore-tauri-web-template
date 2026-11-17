//! A menu bar of drop-down menus. The app describes the menus and what each item
//! does:
//!
//! ```ignore
//! view! {
//!     MenuBar(menus=vec![
//!         Menu::new("File", [MenuItem::disabled("Open"), MenuItem::new("Quit", quit)]),
//!     ])
//! }
//! ```
//!
//! Styles are in `menu.css`, next to this crate's `Cargo.toml`.

use std::rc::Rc;

use sycamore::prelude::*;

/// One drop-down menu: a label in the bar and the items under it.
#[derive(Clone)]
pub struct Menu {
    label: &'static str,
    items: Vec<MenuItem>,
}

impl Menu {
    pub fn new(label: &'static str, items: impl IntoIterator<Item = MenuItem>) -> Self {
        Self {
            label,
            items: items.into_iter().collect(),
        }
    }
}

/// What a menu item shows and does. Items without an action are shown disabled.
#[derive(Clone)]
pub struct MenuItem {
    label: &'static str,
    action: Option<Rc<dyn Fn()>>,
}

impl MenuItem {
    pub fn new(label: &'static str, action: impl Fn() + 'static) -> Self {
        Self {
            label,
            action: Some(Rc::new(action)),
        }
    }

    pub fn disabled(label: &'static str) -> Self {
        Self { label, action: None }
    }
}

#[component(inline_props)]
pub fn MenuBar(menus: Vec<Menu>) -> View {
    let open_menu = create_signal(None::<usize>);
    let menus: Vec<View> = menus
        .into_iter()
        .enumerate()
        .map(|(id, m)| menu(open_menu, id, m))
        .collect();

    view! {
        nav(class="menu-bar") { (menus) }
    }
}

fn menu(open_menu: Signal<Option<usize>>, id: usize, menu: Menu) -> View {
    let is_open = move || open_menu.get() == Some(id);
    let items = menu.items;

    view! {
        div(class="menu") {
            button(
                class=if is_open() { "menu-button open" } else { "menu-button" },
                on:click=move |_| {
                    open_menu.set(if open_menu.get() == Some(id) { None } else { Some(id) });
                },
            ) { (menu.label) }
            (if is_open() {
                let buttons: Vec<View> = items
                    .iter()
                    .cloned()
                    .map(|MenuItem { label, action }| {
                        let disabled = action.is_none();
                        view! {
                            button(
                                disabled=disabled,
                                on:click=move |_| {
                                    open_menu.set(None);
                                    if let Some(action) = &action {
                                        action();
                                    }
                                },
                            ) { (label) }
                        }
                    })
                    .collect();
                view! {
                    div(class="menu-dropdown") { (buttons) }
                }
            } else {
                View::new()
            })
        }
    }
}
