//! Docking: tabbed panels in a tree of resizable splits. Drag a tab onto another
//! panel to merge it as a tab (centre / tab strip) or split that panel (edges).
//!
//! The app supplies the panels by implementing [`Panel`] (from the `panel` crate),
//! describes where they start with a [`DockTree`], and renders a [`Dock`] for the
//! resulting [`DockLayout`]:
//!
//! ```ignore
//! let layout = DockLayout::new(DockTree::row([
//!     (1.0, DockTree::leaf([MyPanel::Files])),
//!     (3.0, DockTree::leaf([MyPanel::Editor, MyPanel::Preview])),
//! ]));
//! view! { Dock(layout=layout) }
//! ```
//!
//! Panel contents are rebuilt whenever the layout changes, so panels must not own
//! long-lived state: keep it in signals provided as context above the [`Dock`].
//!
//! Styles are in `dock.css`, next to this crate's `Cargo.toml`.

use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::PointerEvent;

pub use panel::Panel;

/// Distance (in pixels) a pointer must move from a tab before it counts as a drag rather than a click.
const DRAG_THRESHOLD: f64 = 4.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SplitDir {
    Row,
    Column,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Zone {
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

impl Zone {
    fn as_str(self) -> &'static str {
        match self {
            Zone::Center => "center",
            Zone::Left => "left",
            Zone::Right => "right",
            Zone::Top => "top",
            Zone::Bottom => "bottom",
        }
    }

    /// Arrow glyph shown for this zone's drop indicator. Center has no icon -- it's merged
    /// via the tab strip or by dropping in the (otherwise unmarked) middle of the leaf.
    fn icon(self) -> &'static str {
        match self {
            Zone::Center => "",
            Zone::Left => "\u{25c0}",
            Zone::Right => "\u{25b6}",
            Zone::Top => "\u{25b2}",
            Zone::Bottom => "\u{25bc}",
        }
    }
}

#[derive(Clone)]
enum DockNode<P> {
    Leaf {
        id: u32,
        tabs: Vec<P>,
        active: P,
    },
    Split {
        dir: SplitDir,
        /// Each child paired with its flex-grow weight, so the two panes either side of a
        /// resize handle can be adjusted without disturbing their other siblings.
        children: Vec<(f64, DockNode<P>)>,
    },
}

#[derive(Clone, Copy)]
struct DragPayload<P> {
    panel: P,
}

/// Snapshot taken when a resize handle is grabbed, so the drag can compute new weights purely
/// from how far the pointer has moved, without re-measuring the DOM every frame.
#[derive(Clone, Copy)]
struct ResizeState {
    start_client: f64,
    size_a: f64,
    size_b: f64,
    weight_a: f64,
    weight_b: f64,
}

/// Minimum on-screen size (px) a pane is allowed to shrink to via a resize handle.
const MIN_PANE_SIZE: f64 = 80.0;

/// A starting layout for a [`DockLayout`]: panels grouped as tabs ([`DockTree::leaf`])
/// in rows and columns of weighted, resizable splits.
#[derive(Clone)]
pub struct DockTree<P>(DockNode<P>);

impl<P: Panel> DockTree<P> {
    /// A group of tabbed panels, the first one showing. Panics if `tabs` is empty.
    pub fn leaf(tabs: impl IntoIterator<Item = P>) -> Self {
        let tabs: Vec<P> = tabs.into_iter().collect();
        let active = *tabs.first().expect("a dock leaf needs at least one panel");
        // Ids are assigned by `DockLayout::new`.
        Self(DockNode::Leaf { id: 0, tabs, active })
    }

    /// Side-by-side children, each paired with its share (flex-grow weight) of the width.
    pub fn row(children: impl IntoIterator<Item = (f64, DockTree<P>)>) -> Self {
        Self::split(SplitDir::Row, children)
    }

    /// Stacked children, each paired with its share (flex-grow weight) of the height.
    pub fn column(children: impl IntoIterator<Item = (f64, DockTree<P>)>) -> Self {
        Self::split(SplitDir::Column, children)
    }

    fn split(dir: SplitDir, children: impl IntoIterator<Item = (f64, DockTree<P>)>) -> Self {
        Self(DockNode::Split {
            dir,
            children: children.into_iter().map(|(w, c)| (w, c.0)).collect(),
        })
    }
}

/// A dock's live layout. It is `Copy`, so it can be shared (for instance as context)
/// with things outside the [`Dock`], such as a menu that resets it.
pub struct DockLayout<P: 'static> {
    tree: Signal<DockNode<P>>,
    initial: Signal<DockNode<P>>,
    next_id: Signal<u32>,
}

impl<P> Clone for DockLayout<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P> Copy for DockLayout<P> {}

impl<P: Panel> DockLayout<P> {
    pub fn new(initial: DockTree<P>) -> Self {
        let mut root = initial.0;
        let mut next_id = 0;
        assign_ids(&mut root, &mut next_id);
        Self {
            tree: create_signal(root.clone()),
            initial: create_signal(root),
            next_id: create_signal(next_id),
        }
    }

    /// Puts every panel back where it started.
    pub fn reset(&self) {
        self.tree.set(self.initial.get_clone());
    }
}

fn assign_ids<P>(node: &mut DockNode<P>, next_id: &mut u32) {
    match node {
        DockNode::Leaf { id, .. } => {
            *id = *next_id;
            *next_id += 1;
        }
        DockNode::Split { children, .. } => {
            for (_, child) in children.iter_mut() {
                assign_ids(child, next_id);
            }
        }
    }
}

struct DockCtx<P: 'static> {
    tree: Signal<DockNode<P>>,
    dragging: Signal<Option<DragPayload<P>>>,
    drag_start: Signal<Option<(P, u32, f64, f64)>>,
    drag_pos: Signal<(f64, f64)>,
    hover: Signal<Option<(u32, Zone)>>,
    next_id: Signal<u32>,
}

impl<P> Clone for DockCtx<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P> Copy for DockCtx<P> {}

#[component(inline_props)]
pub fn Dock<P: Panel>(layout: DockLayout<P>) -> View {
    let ctx = DockCtx {
        tree: layout.tree,
        dragging: create_signal(None),
        drag_start: create_signal(None),
        drag_pos: create_signal((0.0, 0.0)),
        hover: create_signal(None),
        // Ids only need to be unique within the current tree; the counter never
        // restarts, so ids handed out before a reset can't clash with the initial ones.
        next_id: layout.next_id,
    };

    let ctx_tree = ctx.clone();
    let ctx_ghost = ctx.clone();

    view! {
        div(class="dock") {
            div(class="dock-body") {
                (render_tree(&ctx_tree.tree.get_clone(), Vec::new(), ctx_tree.clone()))
            }
            (if let Some(payload) = ctx_ghost.dragging.get() {
                let (x, y) = ctx_ghost.drag_pos.get();
                view! {
                    div(class="drag-ghost", style=format!("left: {}px; top: {}px;", x + 14.0, y + 10.0)) {
                        (payload.panel.title())
                    }
                }
            } else {
                View::new()
            })
        }
    }
}

fn render_tree<P: Panel>(node: &DockNode<P>, path: Vec<usize>, ctx: DockCtx<P>) -> View {
    match node {
        DockNode::Split { dir, children } => render_split(*dir, children, path, ctx),
        DockNode::Leaf { id, tabs, active } => render_leaf(*id, tabs.clone(), *active, ctx),
    }
}

fn render_split<P: Panel>(dir: SplitDir, children: &[(f64, DockNode<P>)], path: Vec<usize>, ctx: DockCtx<P>) -> View {
    let class = match dir {
        SplitDir::Row => "split split-row",
        SplitDir::Column => "split split-column",
    };

    let mut items: Vec<View> = Vec::with_capacity(children.len() * 2);
    for (i, (weight, child)) in children.iter().enumerate() {
        let weight = *weight;
        let mut child_path = path.clone();
        child_path.push(i);
        let rendered_child = render_tree(child, child_path, ctx.clone());
        items.push(view! {
            div(class="split-item", style=format!("flex: {weight} 1 0%;"), data-weight=weight.to_string()) {
                (rendered_child)
            }
        });
        if i + 1 < children.len() {
            items.push(resize_handle(dir, path.clone(), i, ctx.clone()));
        }
    }

    view! {
        div(class=class) { (items) }
    }
}

fn resize_handle<P: Panel>(dir: SplitDir, split_path: Vec<usize>, pair_index: usize, ctx: DockCtx<P>) -> View {
    let class = match dir {
        SplitDir::Row => "resize-handle resize-handle-row",
        SplitDir::Column => "resize-handle resize-handle-column",
    };
    let resize_state: Signal<Option<ResizeState>> = create_signal(None);

    view! {
        div(
            class=class,
            on:pointerdown=move |e: PointerEvent| {
                let Some(target) = e.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) else {
                    return;
                };
                let _ = target.set_pointer_capture(e.pointer_id());
                if let Some(state) = measure_resize_state(&target, dir, e.client_x() as f64, e.client_y() as f64) {
                    resize_state.set(Some(state));
                }
            },
            on:pointermove=move |e: PointerEvent| {
                let Some(state) = resize_state.get() else { return; };
                let Some(target) = e.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) else {
                    return;
                };
                // Live feedback is a direct DOM write, not a tree update: updating the
                // reactive tree on every pointermove would tear down and recreate this very
                // handle (and its drag state) mid-gesture, killing the drag after one step.
                preview_resize(&target, dir, state, e.client_x() as f64, e.client_y() as f64);
            },
            on:pointerup=move |e: PointerEvent| {
                let Some(state) = resize_state.get() else { return; };
                resize_state.set(None);
                commit_resize(&ctx, &split_path, pair_index, dir, state, e.client_x() as f64, e.client_y() as f64);
            },
        ) {}
    }
}

fn measure_resize_state(handle: &web_sys::Element, dir: SplitDir, client_x: f64, client_y: f64) -> Option<ResizeState> {
    let prev = handle.previous_element_sibling()?;
    let next = handle.next_element_sibling()?;
    let prev_rect = prev.get_bounding_client_rect();
    let next_rect = next.get_bounding_client_rect();
    let (size_a, size_b, start_client) = match dir {
        SplitDir::Row => (prev_rect.width(), next_rect.width(), client_x),
        SplitDir::Column => (prev_rect.height(), next_rect.height(), client_y),
    };
    let weight_a: f64 = prev.get_attribute("data-weight")?.parse().ok()?;
    let weight_b: f64 = next.get_attribute("data-weight")?.parse().ok()?;
    Some(ResizeState {
        start_client,
        size_a,
        size_b,
        weight_a,
        weight_b,
    })
}

/// Computes the (weight_a, weight_b) pair that keeps weight_a + weight_b constant -- so only
/// the two panes either side of the handle change size, leaving every other sibling untouched.
fn compute_new_weights(dir: SplitDir, state: ResizeState, client_x: f64, client_y: f64) -> (f64, f64) {
    let current = match dir {
        SplitDir::Row => client_x,
        SplitDir::Column => client_y,
    };
    let delta = (current - state.start_client).clamp(MIN_PANE_SIZE - state.size_a, state.size_b - MIN_PANE_SIZE);
    let new_size_a = state.size_a + delta;
    let new_size_b = state.size_b - delta;
    let scale = (state.size_a + state.size_b) / (state.weight_a + state.weight_b);
    if scale <= 0.0 {
        return (state.weight_a, state.weight_b);
    }
    ((new_size_a / scale).max(0.01), (new_size_b / scale).max(0.01))
}

fn preview_resize(handle: &web_sys::Element, dir: SplitDir, state: ResizeState, client_x: f64, client_y: f64) {
    let (weight_a, weight_b) = compute_new_weights(dir, state, client_x, client_y);
    if let Some(prev) = handle.previous_element_sibling() {
        let _ = prev.set_attribute("style", &format!("flex: {weight_a} 1 0%;"));
    }
    if let Some(next) = handle.next_element_sibling() {
        let _ = next.set_attribute("style", &format!("flex: {weight_b} 1 0%;"));
    }
}

fn commit_resize<P: Panel>(
    ctx: &DockCtx<P>,
    split_path: &[usize],
    pair_index: usize,
    dir: SplitDir,
    state: ResizeState,
    client_x: f64,
    client_y: f64,
) {
    let (weight_a, weight_b) = compute_new_weights(dir, state, client_x, client_y);
    ctx.tree.update(|root| {
        if let Some(children) = split_children_at(root, split_path) {
            if let Some(pair) = children.get_mut(pair_index) {
                pair.0 = weight_a;
            }
            if let Some(pair) = children.get_mut(pair_index + 1) {
                pair.0 = weight_b;
            }
        }
    });
}

fn split_children_at<'a, P>(node: &'a mut DockNode<P>, path: &[usize]) -> Option<&'a mut Vec<(f64, DockNode<P>)>> {
    match path {
        [] => match node {
            DockNode::Split { children, .. } => Some(children),
            DockNode::Leaf { .. } => None,
        },
        [first, rest @ ..] => match node {
            DockNode::Split { children, .. } => {
                let (_, child) = children.get_mut(*first)?;
                split_children_at(child, rest)
            }
            DockNode::Leaf { .. } => None,
        },
    }
}

fn render_leaf<P: Panel>(id: u32, tabs: Vec<P>, active: P, ctx: DockCtx<P>) -> View {
    let ctx_tabs = ctx.clone();
    let ctx_zones = ctx.clone();

    view! {
        div(class="leaf") {
            // Dropping anywhere on the tab strip (including on top of another tab) always
            // merges as a tab there -- splitting from the tab strip would be surprising.
            div(class="tab-strip", data-drop-leaf=id.to_string(), data-drop-zone="center") {
                (tabs.iter().copied().map(|panel_id| {
                    let ctx = ctx_tabs.clone();
                    let ctx_down = ctx.clone();
                    let ctx_move = ctx.clone();
                    let ctx_up = ctx.clone();
                    view! {
                        div(
                            class=if panel_id == active { "tab active" } else { "tab" },
                            on:pointerdown=move |e: PointerEvent| {
                                if let Some(target) = e.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                                    let _ = target.set_pointer_capture(e.pointer_id());
                                }
                                let pos = (e.client_x() as f64, e.client_y() as f64);
                                ctx_down.drag_start.set(Some((panel_id, id, pos.0, pos.1)));
                                ctx_down.drag_pos.set(pos);
                            },
                            on:pointermove=move |e: PointerEvent| {
                                let Some((panel, _, sx, sy)) = ctx_move.drag_start.get() else { return; };
                                let (cx, cy) = (e.client_x() as f64, e.client_y() as f64);
                                ctx_move.drag_pos.set((cx, cy));
                                if ctx_move.dragging.get().is_none() {
                                    if (cx - sx).abs() > DRAG_THRESHOLD || (cy - sy).abs() > DRAG_THRESHOLD {
                                        ctx_move.dragging.set(Some(DragPayload { panel }));
                                    } else {
                                        return;
                                    }
                                }
                                ctx_move.hover.set(hit_test(cx, cy));
                            },
                            on:pointerup=move |_: PointerEvent| {
                                let was_dragging = ctx_up.dragging.get();
                                let start = ctx_up.drag_start.get();
                                ctx_up.drag_start.set(None);
                                ctx_up.dragging.set(None);
                                let hover_target = ctx_up.hover.get();
                                ctx_up.hover.set(None);
                                match (was_dragging, hover_target) {
                                    (Some(payload), Some((leaf_id, zone))) => {
                                        handle_drop(&ctx_up, payload, leaf_id, zone);
                                    }
                                    (None, _) => {
                                        if let Some((panel, leaf_id, _, _)) = start {
                                            set_active(&ctx_up, leaf_id, panel);
                                        }
                                    }
                                    _ => {}
                                }
                            },
                        ) {
                            (panel_id.title())
                        }
                    }
                }).collect::<Vec<_>>())
            }
            div(class="leaf-content") {
                (active.content())
                (drop_zones(id, ctx_zones.clone()))
            }
        }
    }
}

fn drop_zones<P: Panel>(leaf_id: u32, ctx: DockCtx<P>) -> View {
    if ctx.dragging.get().is_none() {
        return View::new();
    }
    view! {
        div(class="drop-overlay") {
            (drop_zone(leaf_id, Zone::Left, ctx.hover))
            (drop_zone(leaf_id, Zone::Right, ctx.hover))
            (drop_zone(leaf_id, Zone::Top, ctx.hover))
            (drop_zone(leaf_id, Zone::Bottom, ctx.hover))
            (drop_zone(leaf_id, Zone::Center, ctx.hover))
        }
    }
}

fn drop_zone(leaf_id: u32, zone: Zone, hover: Signal<Option<(u32, Zone)>>) -> View {
    let base_class = format!("drop-zone {}", zone.as_str());
    view! {
        div(
            class=if hover.get() == Some((leaf_id, zone)) { format!("{base_class} hover") } else { base_class.clone() },
            data-drop-leaf=leaf_id.to_string(),
            data-drop-zone=zone.as_str(),
        ) {
            (if !zone.icon().is_empty() {
                view! { div(class="drop-icon") { (zone.icon()) } }
            } else {
                View::new()
            })
        }
    }
}

fn hit_test(client_x: f64, client_y: f64) -> Option<(u32, Zone)> {
    let mut node = document().element_from_point(client_x as f32, client_y as f32);
    while let Some(el) = node {
        if let Some(zone_str) = el.get_attribute("data-drop-zone") {
            let leaf_id: u32 = el.get_attribute("data-drop-leaf")?.parse().ok()?;
            let zone = match zone_str.as_str() {
                "center" => Zone::Center,
                "left" => Zone::Left,
                "right" => Zone::Right,
                "top" => Zone::Top,
                "bottom" => Zone::Bottom,
                _ => return None,
            };
            return Some((leaf_id, zone));
        }
        node = el.parent_element();
    }
    None
}

fn set_active<P: Panel>(ctx: &DockCtx<P>, leaf_id: u32, panel: P) {
    ctx.tree.update(|root| set_active_node(root, leaf_id, panel));
}

fn set_active_node<P: Panel>(node: &mut DockNode<P>, leaf_id: u32, panel: P) {
    match node {
        DockNode::Leaf { id, active, .. } if *id == leaf_id => *active = panel,
        DockNode::Leaf { .. } => {}
        DockNode::Split { children, .. } => {
            for (_, child) in children.iter_mut() {
                set_active_node(child, leaf_id, panel);
            }
        }
    }
}

fn handle_drop<P: Panel>(ctx: &DockCtx<P>, payload: DragPayload<P>, target_leaf: u32, zone: Zone) {
    // Dropping a leaf's only tab back onto itself would delete the target before it could be
    // re-inserted into, silently losing the panel -- skip instead.
    let skip = ctx.tree.with(|root| {
        matches!(
            find_leaf(root, target_leaf),
            Some(DockNode::Leaf { tabs, .. }) if tabs.len() == 1 && tabs[0] == payload.panel
        )
    });
    if skip {
        return;
    }
    move_panel(ctx, payload.panel, target_leaf, zone);
}

fn move_panel<P: Panel>(ctx: &DockCtx<P>, panel: P, target_leaf: u32, zone: Zone) {
    ctx.tree.update(|root| {
        let removed = remove_panel(root, panel).unwrap_or_else(|| {
            let id = ctx.next_id.get();
            ctx.next_id.set(id + 1);
            DockNode::Leaf {
                id,
                tabs: vec![],
                active: panel,
            }
        });
        *root = insert_panel(&removed, target_leaf, zone, panel, ctx.next_id);
    });
}

fn find_leaf<P>(node: &DockNode<P>, id: u32) -> Option<&DockNode<P>> {
    match node {
        DockNode::Leaf { id: lid, .. } if *lid == id => Some(node),
        DockNode::Leaf { .. } => None,
        DockNode::Split { children, .. } => children.iter().find_map(|(_, c)| find_leaf(c, id)),
    }
}

fn remove_panel<P: Panel>(node: &DockNode<P>, panel: P) -> Option<DockNode<P>> {
    match node {
        DockNode::Leaf { id, tabs, active } => {
            let mut tabs = tabs.clone();
            tabs.retain(|p| *p != panel);
            if tabs.is_empty() {
                None
            } else {
                let active = if *active == panel { tabs[0] } else { *active };
                Some(DockNode::Leaf { id: *id, tabs, active })
            }
        }
        DockNode::Split { dir, children } => {
            let new_children: Vec<(f64, DockNode<P>)> = children
                .iter()
                .filter_map(|(w, c)| remove_panel(c, panel).map(|nc| (*w, nc)))
                .collect();
            match new_children.len() {
                0 => None,
                1 => Some(new_children.into_iter().next().unwrap().1),
                _ => Some(DockNode::Split {
                    dir: *dir,
                    children: new_children,
                }),
            }
        }
    }
}

fn insert_panel<P: Panel>(node: &DockNode<P>, target_id: u32, zone: Zone, panel: P, next_id: Signal<u32>) -> DockNode<P> {
    match node {
        DockNode::Leaf { id, tabs, .. } => {
            if *id != target_id {
                return node.clone();
            }
            match zone {
                Zone::Center => {
                    let mut tabs = tabs.clone();
                    tabs.push(panel);
                    DockNode::Leaf {
                        id: *id,
                        tabs,
                        active: panel,
                    }
                }
                Zone::Left | Zone::Right | Zone::Top | Zone::Bottom => {
                    let new_id = next_id.get();
                    next_id.set(new_id + 1);
                    let new_leaf = DockNode::Leaf {
                        id: new_id,
                        tabs: vec![panel],
                        active: panel,
                    };
                    let existing = node.clone();
                    let dir = if matches!(zone, Zone::Left | Zone::Right) {
                        SplitDir::Row
                    } else {
                        SplitDir::Column
                    };
                    let children = if matches!(zone, Zone::Left | Zone::Top) {
                        vec![(1.0, new_leaf), (1.0, existing)]
                    } else {
                        vec![(1.0, existing), (1.0, new_leaf)]
                    };
                    DockNode::Split { dir, children }
                }
            }
        }
        DockNode::Split { dir, children } => DockNode::Split {
            dir: *dir,
            children: children
                .iter()
                .map(|(w, c)| (*w, insert_panel(c, target_id, zone, panel, next_id)))
                .collect(),
        },
    }
}
