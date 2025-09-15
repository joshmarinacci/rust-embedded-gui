use crate::geom::{Bounds, Point};
use crate::view::View;
use crate::{Action, Callback, DrawEvent, DrawingContext, EventType, GuiEvent, LayoutEvent, Theme};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use hashbrown::HashMap;

#[derive(Debug)]
pub struct Scene<C, F> {
    pub(crate) keys: HashMap<String, View<C, F>>,
    children: HashMap<String, Vec<String>>,
    pub(crate) dirty: bool,
    pub bounds: Bounds,
    pub dirty_rect: Bounds,
    pub root_id: String,
    pub(crate) focused: Option<String>,
}

impl<C, F> Scene<C, F> {
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

    pub fn get_view(&self, name: &str) -> Option<&View<C, F>> {
        self.keys.get(name)
    }
    pub fn get_view_mut(&mut self, name: &str) -> Option<&mut View<C, F>> {
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
    pub fn remove_view(&mut self, name: &str) -> Option<View<C, F>> {
        self.mark_dirty_view(name);
        self.keys.remove(name)
    }
    pub fn new_with_bounds(bounds: Bounds) -> Scene<C, F> {
        let root = View {
            name: "root".to_string(),
            title: "root".to_string(),
            bounds,
            visible: true,
            draw: None,
            input: None,
            state: None,
            layout: None,
            draw2: Some(|e| e.ctx.fill_rect(&e.view.bounds, &e.theme.panel_bg)),
        };
        let root_id = String::from("root");
        let mut keys: HashMap<String, View<C, F>> = HashMap::new();
        keys.insert(root_id.clone(), root);
        Scene {
            bounds,
            keys,
            dirty: true,
            root_id,
            focused: None,
            dirty_rect: bounds,
            children: HashMap::new(),
        }
    }
    pub fn new() -> Scene<C, F> {
        let bounds = Bounds {
            x: 0,
            y: 0,
            w: 200,
            h: 200,
        };
        Self::new_with_bounds(bounds)
    }
    pub fn add_view(&mut self, view: View<C, F>) {
        let name = view.name.clone();
        self.keys.insert(name.clone(), view);
        self.mark_dirty_view(&name);
    }
    pub fn add_view_to_root(&mut self, view: View<C, F>) {
        self.add_view_to_parent(view, &self.root_id.clone());
    }
    pub fn add_view_to_parent(&mut self, view: View<C, F>, parent: &str) {
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

pub fn click_at<C, F>(
    scene: &mut Scene<C, F>,
    handlers: &Vec<Callback<C, F>>,
    pt: Point,
) -> Option<EventResult> {
    // info!("picking at {:?}", pt);
    let targets = pick_at(scene, &pt);
    if let Some(target) = targets.last() {
        // info!("doing the target {}", target);
        let mut event: GuiEvent<C, F> = GuiEvent {
            scene,
            target,
            event_type: EventType::Tap(pt),
            action: None,
        };
        if let Some(view) = event.scene.get_view(target) {
            // info!("got the view {:?}", view.name);
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

pub fn type_at_focused<C, F>(
    scene: &mut Scene<C, F>,
    handlers: &Vec<Callback<C, F>>,
    key: u8,
) -> Option<EventResult> {
    if scene.focused.is_some() {
        let focused = scene.focused.as_ref().unwrap().clone();
        let mut event: GuiEvent<C, F> = GuiEvent {
            scene,
            target: &focused,
            event_type: EventType::Keyboard(key),
            action: None,
        };
        if let Some(view) = event.scene.get_view(&focused) {
            if let Some(input) = view.input {
                event.action = input(&mut event);
            }
            for cb in handlers {
                cb(&mut event);
            }
            if let Some(action) = event.action {
                return Some((focused, action));
            }
        }
    }
    None
}

pub fn scroll_at_focused<C, F>(
    scene: &mut Scene<C, F>,
    handlers: &Vec<Callback<C, F>>,
    dx: i32,
    dy: i32,
) -> Option<EventResult> {
    if scene.focused.is_some() {
        let focused = scene.focused.as_ref().unwrap().clone();
        let mut event: GuiEvent<C, F> = GuiEvent {
            scene,
            target: &focused,
            event_type: EventType::Scroll(dx, dy),
            action: None,
        };
        if let Some(view) = event.scene.get_view(&focused) {
            if let Some(input) = view.input {
                event.action = input(&mut event);
            }
            for cb in handlers {
                cb(&mut event);
            }
            if let Some(action) = event.action {
                return Some((focused, action));
            }
        }
    }
    None
}

pub fn action_at_focused<C, F>(
    scene: &mut Scene<C, F>,
    handlers: &Vec<Callback<C, F>>,
) -> Option<EventResult> {
    if scene.focused.is_some() {
        let focused = scene.focused.as_ref().unwrap().clone();
        let mut event: GuiEvent<C, F> = GuiEvent {
            scene,
            target: &focused,
            event_type: EventType::Action(),
            action: None,
        };
        if let Some(view) = &event.scene.get_view(&focused) {
            if let Some(input) = view.input {
                event.action = input(&mut event);
            }
            for cb in handlers {
                cb(&mut event);
            }
            if let Some(action) = event.action {
                return Some((focused, action));
            }
        }
    }
    None
}

pub fn pick_at<C, F>(scene: &mut Scene<C, F>, pt: &Point) -> Vec<String> {
    pick_at_view(scene, pt, &scene.root_id)
}

fn pick_at_view<C, F>(scene: &Scene<C, F>, pt: &Point, name: &str) -> Vec<String> {
    let mut coll: Vec<String> = vec![];
    if let Some(view) = scene.keys.get(name) {
        if view.bounds.contains(pt) && view.visible {
            coll.push(view.name.clone());
            let pt2 = Point {
                x: pt.x - view.bounds.x,
                y: pt.y - view.bounds.y,
            };
            for kid in scene.get_children(&view.name) {
                let mut coll2 = pick_at_view(scene, &pt2, &kid);
                coll.append(&mut coll2);
            }
        }
    }
    coll
}

pub fn draw_scene<C, F>(
    scene: &mut Scene<C, F>,
    ctx: &mut dyn DrawingContext<C, F>,
    theme: &Theme<C, F>,
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

fn draw_view<C, F>(
    scene: &mut Scene<C, F>,
    ctx: &mut dyn DrawingContext<C, F>,
    theme: &Theme<C, F>,
    name: &str,
) {
    let focused = &scene.focused.clone();
    let bounds = &scene.bounds.clone();
    if let Some(view) = scene.get_view_mut(name) {
        if view.visible {
            if let Some(draw) = view.draw {
                draw(view, ctx, theme);
            }
            if let Some(draw2) = view.draw2 {
                let mut de: DrawEvent<C, F> = DrawEvent {
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
        for kid in scene.get_children(&view.name) {
            draw_view(scene, ctx, theme, &kid);
        }
    }
}

pub fn layout_scene<C, F>(scene: &mut Scene<C, F>) {
    let root_id = scene.root_id.clone();
    layout_view(scene, &root_id);
}

fn layout_view<C, F>(scene: &mut Scene<C, F>, name: &str) {
    let mut evt: LayoutEvent<C, F> = LayoutEvent {
        scene,
        target: name,
    };
    if let Some(form) = evt.scene.get_view(name) {
        if let Some(layout) = &form.layout {
            layout(&mut evt);
        }
    }
    if let Some(view) = scene.get_view(name) {
        for kid in scene.get_children(&view.name) {
            layout_view(scene, &kid);
        }
    }
}
