use alloc::string::{String, ToString};
use hashbrown::HashMap;
use alloc::vec;
use alloc::vec::Vec;
use crate::DrawingContext;
use crate::geom::Bounds;
use crate::view::View;

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
            draw2: Some(|e|{
                e.ctx.fill_rect(&e.view.bounds, &e.theme.panel_bg)
            }),
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