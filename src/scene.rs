use crate::geom::{Bounds, Point};
use crate::gfx::DrawingContext;
use crate::view::{View, ViewId};
use crate::{Action, Callback, DrawEvent, EventType, GuiEvent, LayoutEvent, LayoutFn, Theme};
use alloc::vec::Vec;
use alloc::{format, vec};
use hashbrown::HashMap;
use log::{info, warn};

#[derive(Debug)]
pub struct Scene {
    pub(crate) keys: HashMap<ViewId, View>,
    children: HashMap<ViewId, Vec<ViewId>>,
    pub(crate) dirty: bool,
    pub bounds: Bounds,
    pub dirty_rect: Bounds,
    pub root_id: ViewId,
    pub(crate) focused: Option<ViewId>,
    pub layout_dirty: bool,
}

impl Scene {
    pub fn dump(&self) {
        self.dump_view(&self.root_id.clone(), "");
    }
    fn dump_view(&self, id: &ViewId, indent: &str) {
        if let Some(view) = self.get_view(&id) {
            info!("{indent}{id} ---");
            // info!("{indent}  padding {}", view.padding);
            info!("{indent}  bounds  {}", view.bounds);
            info!("{indent}  h = {:?} {:?}", view.h_flex, view.h_align);
            info!("{indent}  v = {:?} {:?}", view.v_flex, view.v_align);
        }
        let kids = self.get_children_ids(id);
        for kid in kids {
            self.dump_view(&kid, &format!("{indent}    "));
        }
    }
}

impl Scene {
    pub fn root_id(&self) -> ViewId {
        self.root_id
    }
    pub fn set_focused(&mut self, name: &ViewId) {
        if self.focused.is_some() {
            let fo = self.focused.as_ref().unwrap().clone();
            self.mark_dirty_view(&fo);
        }
        self.focused = Some(name.clone());
        self.mark_dirty_view(name);
    }
    pub fn get_focused(&self) -> Option<ViewId> {
        self.focused.clone()
    }
    pub fn is_focused(&self, name: &ViewId) -> bool {
        self.focused.as_ref().is_some_and(|focused| focused == name)
    }
    pub fn is_visible(&self, name: &ViewId) -> bool {
        if let Some(view) = self.get_view(name) {
            view.visible
        } else {
            false
        }
    }
    pub fn show_view(&mut self, name: &ViewId) {
        if let Some(view) = self.get_view_mut(name) {
            view.visible = true;
        }
        self.mark_dirty_view(name);
    }
    pub fn hide_view(&mut self, name: &ViewId) {
        if let Some(view) = self.get_view_mut(name) {
            view.visible = false;
        }
        self.mark_dirty_view(name);
    }
    pub fn mark_dirty_all(&mut self) {
        self.dirty_rect = self.bounds;
        self.dirty = true;
    }
    pub fn mark_dirty_view(&mut self, name: &ViewId) {
        if let Some(view) = self.get_view(name) {
            self.dirty_rect = self.dirty_rect.union(view.bounds);
            self.dirty = true;
        }
        self.mark_dirty_all();
    }
    pub fn mark_layout_dirty(&mut self) {
        self.layout_dirty = true;
        self.mark_dirty_all();
    }
    pub fn remove_child(&mut self, parent: &ViewId, child: &ViewId) {
        if let Some(children) = self.children.get_mut(parent) {
            if let Some(n) = children.iter().position(|name| name == child) {
                children.remove(n);
            }
        }
    }
    pub fn add_child(&mut self, parent: &ViewId, child: &ViewId) {
        if !self.children.contains_key(parent) {
            self.children.insert(parent.clone(), vec![]);
        }
        if let Some(children) = self.children.get_mut(parent) {
            children.push(child.clone());
        }
    }
    pub fn get_children_ids(&self, name: &ViewId) -> Vec<ViewId> {
        if let Some(children) = self.children.get(name) {
            children.clone()
        } else {
            Vec::new()
        }
    }
    pub fn get_children_ids_filtered(&self, id: &ViewId, cb: fn(&View) -> bool) -> Vec<ViewId> {
        self.get_children_ids(id)
            .iter()
            .map(|kid| self.get_view(kid))
            .flatten()
            .filter(|v| cb(v)) // WORKS
            // .filter(cb) // DOESN'T WORK
            .map(|v| v.name.clone())
            .collect()
    }

    pub(crate) fn has_view(&self, name: &ViewId) -> bool {
        self.keys.contains_key(name)
    }
    pub fn get_view(&self, name: &ViewId) -> Option<&View> {
        self.keys.get(name)
    }
    pub fn get_view_mut(&mut self, name: &ViewId) -> Option<&mut View> {
        self.keys.get_mut(name)
    }
    pub fn get_view_state<T: 'static>(&mut self, name: &ViewId) -> Option<&mut T> {
        if let Some(view) = self.get_view_mut(name) {
            if let Some(view) = &mut view.state {
                return view.downcast_mut::<T>();
            }
        }
        None
    }
    pub fn get_view_layout(&mut self, name: &ViewId) -> Option<LayoutFn> {
        if let Some(view) = self.get_view_mut(name) {
            return view.layout;
        }
        None
    }
    pub(crate) fn get_view_bounds(&self, p0: &ViewId) -> Bounds {
        if let Some(view) = self.get_view(p0) {
            view.bounds.clone()
        } else {
            Bounds::new(-99, -99, -99, -99)
        }
    }

    pub(crate) fn viewcount(&self) -> usize {
        self.keys.len()
    }
    pub fn remove_view(&mut self, name: &ViewId) -> Option<View> {
        self.mark_dirty_view(name);
        self.keys.remove(name)
    }
    pub fn new_with_bounds(bounds: Bounds) -> Scene {
        let root_id = ViewId::new("root");
        let root = View {
            name: root_id.clone(),
            title: root_id.as_str().into(),
            bounds,
            visible: true,
            input: None,
            state: None,
            layout: Some(layout_root_panel),
            draw: Some(|e| e.ctx.fill_rect(&e.view.bounds, &e.theme.panel_bg)),
            ..Default::default()
        };
        let mut keys: HashMap<ViewId, View> = HashMap::new();
        keys.insert(root_id.clone(), root);
        Scene {
            bounds,
            keys,
            dirty: true,
            layout_dirty: true,
            root_id,
            focused: None,
            dirty_rect: bounds,
            children: HashMap::new(),
        }
    }
    pub fn new() -> Scene {
        let bounds = Bounds::new(0, 0, 200, 200);
        Self::new_with_bounds(bounds)
    }
    pub fn add_view(&mut self, view: View) {
        let name = view.name.clone();
        if self.keys.contains_key(&name) {
            warn!("might be adding duplicate view key {name}");
        }
        self.keys.insert(name.clone(), view);
        self.mark_dirty_view(&name);
    }
    pub fn add_view_to_root(&mut self, view: View) {
        self.add_view_to_parent(view, &self.root_id.clone());
    }
    pub fn add_view_to_parent(&mut self, view: View, parent: &ViewId) {
        if !self.children.contains_key(parent) {
            self.children.insert(parent.clone(), vec![]);
        }
        if let Some(children) = self.children.get_mut(parent) {
            children.push(view.name.clone());
        }
        self.add_view(view);
    }
    pub fn remove_parent_and_children(&mut self, name: &ViewId) {
        let kids = self.get_children_ids(name);
        for kid in kids {
            self.remove_view(&kid);
            self.remove_child(name, &kid);
        }
        self.remove_view(name);
    }
}

fn layout_root_panel(pass: &mut LayoutEvent) {
    if let Some(view) = pass.scene.get_view_mut(&pass.target) {
        view.bounds.size.w = pass.space.w;
        view.bounds.size.h = pass.space.h;
    }
    for kid in &pass.scene.get_children_ids(&pass.target) {
        pass.layout_child(kid, pass.space);
    }
}

pub type EventResult = (ViewId, Action);

pub fn click_at(scene: &mut Scene, handlers: &Vec<Callback>, pt: Point) -> Option<EventResult> {
    let targets = pick_at(scene, &pt);
    if let Some((target, pt)) = targets.last() {
        let mut event: GuiEvent = GuiEvent {
            scene,
            target,
            event_type: EventType::Tap(pt.clone()),
            action: None,
        };
        if let Some(view) = event.scene.get_view(target) {
            if let Some(input) = view.input {
                event.action = input(&mut event);
            }
        }
        for cb in handlers {
            cb(&mut event);
        }
        if let Some(action) = event.action {
            return Some((target.clone(), action));
        }
    }
    None
}

pub fn event_at_focused(scene: &mut Scene, event_type: &EventType) -> Option<EventResult> {
    if scene.focused.is_some() {
        let focused = scene.focused.as_ref().unwrap().clone();
        let mut event: GuiEvent = GuiEvent {
            scene,
            target: &focused,
            event_type: event_type.clone(),
            action: None,
        };
        if let Some(view) = event.scene.get_view(&focused) {
            if let Some(input) = view.input {
                event.action = input(&mut event);
            }
            if let Some(action) = event.action {
                return Some((focused, action));
            }
        }
    }
    None
}

type Pick = (ViewId, Point);

pub fn pick_at(scene: &mut Scene, pt: &Point) -> Vec<Pick> {
    pick_at_view(scene, pt, &scene.root_id)
}

fn pick_at_view(scene: &Scene, pt: &Point, name: &ViewId) -> Vec<Pick> {
    let mut coll: Vec<Pick> = vec![];
    if let Some(view) = scene.keys.get(name) {
        if view.bounds.contains(pt) && view.visible {
            coll.push((view.name.clone(), pt.clone()));
            let pt2 = pt.subtract(&view.bounds.position);
            for kid in scene.get_children_ids(&view.name) {
                let mut coll2 = pick_at_view(scene, &pt2, &kid);
                coll.append(&mut coll2);
            }
        }
    }
    coll
}

pub fn draw_scene(scene: &mut Scene, ctx: &mut dyn DrawingContext, theme: &Theme) {
    if scene.dirty {
        ctx.fill_rect(&scene.bounds, &theme.panel_bg);
        let name = scene.root_id.clone();
        draw_view(scene, ctx, theme, &name);
        scene.dirty = false;
        scene.dirty_rect = Bounds::new_empty();
    }
}

fn draw_view(scene: &mut Scene, ctx: &mut dyn DrawingContext, theme: &Theme, name: &ViewId) {
    let focused = &scene.focused.clone();
    let bounds = &scene.bounds.clone();
    if let Some(view) = scene.get_view_mut(name) {
        if view.visible {
            if let Some(draw2) = view.draw {
                let mut de: DrawEvent = DrawEvent {
                    theme,
                    view,
                    ctx,
                    focused,
                    bounds,
                };
                draw2(&mut de);
            }
        }
    }
    if let Some(view) = scene.get_view(name) {
        // only draw children if visible
        if view.visible {
            let bounds = view.bounds.clone();
            ctx.translate(&bounds.position);
            for kid in scene.get_children_ids(&view.name) {
                draw_view(scene, ctx, theme, &kid);
            }
            ctx.translate(&bounds.position.negate());
        }
    }
}

pub fn layout_scene(scene: &mut Scene, theme: &Theme) {
    if scene.layout_dirty {
        let mut pass = LayoutEvent {
            target: &scene.root_id(),
            space: scene.bounds.size.clone(),
            scene,
            theme,
        };
        if let Some(layout) = pass.scene.get_view_layout(&pass.scene.root_id()) {
            layout(&mut pass);
        }
        scene.layout_dirty = false;
    }
}

// fn layout_view(scene: &mut Scene, name: &ViewId, theme: &Theme) {
//     let space = scene.bounds.size.clone();
//     let mut evt: LayoutEvent = LayoutEvent {
//         scene,
//         target: name,
//         theme: theme,
//         space: space,
//     };
//     // layout children before the view itself
//     if let Some(view) = evt.scene.get_view(name) {
//         for kid in evt.scene.get_children_ids(&view.name) {
//             layout_view(evt.scene, &kid, theme);
//         }
//     }
//     if let Some(view) = evt.scene.get_view(name) {
//         if let Some(layout) = &view.layout {
//             layout(&mut evt);
//         }
//     }
// }
