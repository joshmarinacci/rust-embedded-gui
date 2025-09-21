use crate::geom::{Bounds, Point};
use crate::view::View;
use crate::{Action, Callback, DrawEvent, DrawingContext, EventType, GuiEvent, LayoutEvent, Theme};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use hashbrown::HashMap;
use log::{info, warn};

#[derive(Debug)]
pub struct Scene {
    pub(crate) keys: HashMap<String, View>,
    children: HashMap<String, Vec<String>>,
    pub(crate) dirty: bool,
    pub bounds: Bounds,
    pub dirty_rect: Bounds,
    pub root_id: String,
    pub(crate) focused: Option<String>,
    pub layout_dirty: bool
}

impl Scene {
    pub fn set_focused(&mut self, name: &str) {
        if self.focused.is_some() {
            let fo = self.focused.as_ref().unwrap().clone();
            self.mark_dirty_view(&fo);
        }
        self.focused = Some(name.into());
        self.mark_dirty_view(name);
    }
    pub fn get_focused(&self) -> Option<String> {
        self.focused.clone()
    }
    pub fn is_focused(&self, name: &str) -> bool {
        self.focused.as_ref().is_some_and(|focused| focused == name)
    }
    pub fn is_visible(&self, name: &str) -> bool {
        if let Some(view) = self.get_view(name) {
            view.visible
        } else {
            false
        }
    }
    pub fn show_view(&mut self, name: &str) {
        if let Some(view) = self.get_view_mut(name) {
            view.visible = true;
        }
        self.mark_dirty_view(name);
    }
    pub fn hide_view(&mut self, name: &str) {
        if let Some(view) = self.get_view_mut(name) {
            view.visible = false;
        }
        self.mark_dirty_view(name);
    }
    pub fn mark_dirty_all(&mut self) {
        self.dirty_rect = self.bounds;
        self.dirty = true;
    }
    pub fn mark_dirty_view(&mut self, name: &str) {
        // info!("Marking dirty view {}", name);
        if let Some(view) = self.get_view(name) {
            self.dirty_rect = self.dirty_rect.union(view.bounds);
            // info!("dirty rect now {:?}", self.dirty_rect);
            self.dirty = true;
        }
    }
    pub fn mark_layout_dirty(&mut self) {
        self.layout_dirty = true;
        self.mark_dirty_all();
    }
    pub fn remove_child(&mut self, parent: &str, child: &str) {
        if let Some(children) = self.children.get_mut(parent) {
            if let Some(n) = children.iter().position(|name| name == child) {
                children.remove(n);
            }
        }
    }
    pub fn add_child(&mut self, parent: &str, child: &str) {
        if !self.children.contains_key(parent) {
            self.children.insert(parent.to_string(), vec![]);
        }
        if let Some(children) = self.children.get_mut(parent) {
            children.push(child.to_string());
        }
    }
    pub fn get_children(&self, name: &str) -> Vec<String> {
        if let Some(children) = self.children.get(name) {
            children.clone()
        } else {
            Vec::new()
        }
    }

    pub fn get_view(&self, name: &str) -> Option<&View> {
        self.keys.get(name)
    }
    pub fn get_view_mut(&mut self, name: &str) -> Option<&mut View> {
        self.keys.get_mut(name)
    }
    pub fn get_view_state<T: 'static>(&mut self, name: &str) -> Option<&mut T> {
        if let Some(view) = self.get_view_mut(name) {
            if let Some(view) = &mut view.state {
                return view.downcast_mut::<T>();
            }
        }
        None
    }

    pub(crate) fn viewcount(&self) -> usize {
        self.keys.len()
    }
    pub fn remove_view(&mut self, name: &str) -> Option<View> {
        self.mark_dirty_view(name);
        self.keys.remove(name)
    }
    pub fn new_with_bounds(bounds: Bounds) -> Scene {
        let root = View {
            name: "root".to_string(),
            title: "root".to_string(),
            bounds,
            visible: true,
            input: None,
            state: None,
            layout: None,
            draw: Some(|e| e.ctx.fill_rect(&e.view.bounds, &e.theme.panel_bg)),
        };
        let root_id = String::from("root");
        let mut keys: HashMap<String, View> = HashMap::new();
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
        let bounds = Bounds {
            x: 0,
            y: 0,
            w: 200,
            h: 200,
        };
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
    pub fn add_view_to_parent(&mut self, view: View, parent: &str) {
        if !self.children.contains_key(parent) {
            self.children.insert(parent.to_string(), vec![]);
        }
        if let Some(children) = self.children.get_mut(parent) {
            children.push(view.name.to_string());
        }
        self.add_view(view);
    }
    pub fn remove_parent_and_children(&mut self, name: &str) {
        let kids = self.get_children(name);
        for kid in kids {
            self.remove_view(&kid);
            self.remove_child(name, &kid);
        }
        self.remove_view(name);
    }
}

pub type EventResult = (String, Action);

pub fn click_at(
    scene: &mut Scene,
    handlers: &Vec<Callback>,
    pt: Point,
) -> Option<EventResult> {
    let targets = pick_at(scene, &pt);
    if let Some(target) = targets.last() {
        let mut event: GuiEvent = GuiEvent {
            scene,
            target,
            event_type: EventType::Tap(pt),
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
            return Some((target.into(), action));
        }
    }
    None
}

pub fn event_at_focused(
    scene: &mut Scene,
    event_type: EventType,
) -> Option<EventResult> {
    if scene.focused.is_some() {
        let focused = scene.focused.as_ref().unwrap().clone();
        let mut event: GuiEvent = GuiEvent {
            scene,
            target: &focused,
            event_type: event_type,
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

pub fn pick_at(scene: &mut Scene, pt: &Point) -> Vec<String> {
    pick_at_view(scene, pt, &scene.root_id)
}

fn pick_at_view(scene: &Scene, pt: &Point, name: &str) -> Vec<String> {
    let mut coll: Vec<String> = vec![];
    if let Some(view) = scene.keys.get(name) {
        if view.bounds.contains(pt) && view.visible {
            coll.push(view.name.clone());
            let pt2 = Point {
                x: pt.x, // - view.bounds.x,
                y: pt.y,// - view.bounds.y,
            };
            for kid in scene.get_children(&view.name) {
                let mut coll2 = pick_at_view(scene, &pt2, &kid);
                coll.append(&mut coll2);
            }
        }
    }
    coll
}

pub fn draw_scene(
    scene: &mut Scene,
    ctx: &mut dyn DrawingContext,
    theme: &Theme
) {
    if scene.dirty {
        // info!(
        //     "draw scene: {} {:?} {:?}",
        //     scene.dirty, scene.bounds, scene.dirty_rect
        // );
        ctx.fill_rect(&scene.bounds, &theme.panel_bg);
        let name = scene.root_id.clone();
        draw_view(scene, ctx, theme, &name);
        scene.dirty = false;
        scene.dirty_rect = Bounds::new_empty();
    }
}

fn draw_view(
    scene: &mut Scene,
    ctx: &mut dyn DrawingContext,
    theme: &Theme,
    name: &str,
) {
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
            ctx.translate(&bounds.position().negate());
            for kid in scene.get_children(&view.name) {
                draw_view(scene, ctx, theme, &kid);
            }
            ctx.translate(&bounds.position());
        }
    }
}

pub fn layout_scene(scene: &mut Scene, theme: &Theme) {
    if scene.layout_dirty {
        let root_id = scene.root_id.clone();
        layout_view(scene, &root_id, theme);
        scene.layout_dirty = false;
    }
}

fn layout_view(scene: &mut Scene, name: &str, theme: &Theme) {
    let mut evt: LayoutEvent = LayoutEvent {
        scene,
        target: name,
        theme: theme,
    };
    // layout children before the view itself
    if let Some(view) = evt.scene.get_view(name) {
        for kid in evt.scene.get_children(&view.name) {
            layout_view(evt.scene, &kid, theme);
        }
    }
    if let Some(view) = evt.scene.get_view(name) {
        if let Some(layout) = &view.layout {
            layout(&mut evt);
        }
    }
}
