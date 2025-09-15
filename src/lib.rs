#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate core;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;
use geom::{Bounds, Point};
use hashbrown::HashMap;
use log::info;

pub mod comps;
pub mod form;
pub mod geom;
pub mod toggle_button;
pub mod toggle_group;

#[derive(Copy, Clone)]
pub enum HAlign {
    Left,
    Center,
    Right,
}
#[derive(Copy, Clone)]
pub enum VAlign {
    Top,
    Center,
    Bottom,
}
pub struct TextStyle<'a, C, F> {
    pub halign: HAlign,
    pub valign: VAlign,
    pub underline: bool,
    pub font: &'a F,
    pub color: &'a C,
}

impl<'a, C, F> TextStyle<'a, C, F> {
    pub fn new(font:&'a F, color: &'a C) -> TextStyle<'a, C,F> {
        TextStyle {
            font,
            color,
            underline: false,
            valign: VAlign::Center,
            halign: HAlign::Left,
        }
    }
    pub fn with_underline(&self, underline: bool) -> Self {
        TextStyle {
            color: self.color,
            font: self.font,
            underline,
            halign: self.halign,
            valign: self.valign
        }
    }
    pub fn with_halign(&self, halign: HAlign) -> Self {
        TextStyle {
            color: self.color,
            font: self.font,
            underline: self.underline,
            halign,
            valign: self.valign
        }
    }
}

pub trait DrawingContext<C, F> {
    fn clear(&mut self, color: &C);
    fn fill_rect(&mut self, bounds: &Bounds, color: &C);
    fn stroke_rect(&mut self, bounds: &Bounds, color: &C);
    fn fill_text(&mut self, bounds: &Bounds, text: &str, style: &TextStyle<C, F>);
}

pub struct DrawEvent<'a, C, F> {
    pub ctx: &'a mut dyn DrawingContext<C, F>,
    pub theme: &'a Theme<C, F>,
    pub focused: &'a Option<String>,
    pub view: &'a View<C, F>,
    pub bounds: &'a Bounds,
}

#[derive(Debug, Clone)]
pub enum Action {
    Generic,
    Command(String),
}
pub type DrawFn<C, F> =
    fn(view: &mut View<C, F>, ctx: &mut dyn DrawingContext<C, F>, theme: &Theme<C, F>);
pub type DrawFn2<C, F> = fn(event: &mut DrawEvent<C, F>);
pub type LayoutFn<C, F> = fn(event: &mut LayoutEvent<C, F>);
pub type InputFn<C, F> = fn(event: &mut GuiEvent<C, F>) -> Option<Action>;

pub struct Theme<C, F> {
    pub bg: C,
    pub fg: C,
    pub panel_bg: C,
    pub font: F,
    pub bold_font: F,
}

#[derive(Debug)]
pub struct View<C, F> {
    pub name: String,
    pub title: String,
    pub bounds: Bounds,
    pub visible: bool,
    pub draw: Option<DrawFn<C, F>>,
    pub input: Option<InputFn<C, F>>,
    pub state: Option<Box<dyn Any>>,
    pub layout: Option<LayoutFn<C, F>>,
    pub draw2: Option<DrawFn2<C, F>>,
}

impl<C, F> View<C, F> {
    pub fn position_at(mut self, x: i32, y: i32) -> View<C, F> {
        self.bounds.x = x;
        self.bounds.y = y;
        self
    }
    pub fn hide(mut self) -> View<C, F> {
        self.visible = false;
        self
    }
    pub fn get_state<T: 'static>(&mut self) -> Option<&mut T> {
        if let Some(view) = &mut self.state {
            return view.downcast_mut::<T>();
        }
        None
    }
}

#[derive(Debug)]
pub struct Scene<C, F> {
    keys: HashMap<String, View<C, F>>,
    children: HashMap<String, Vec<String>>,
    dirty: bool,
    pub bounds: Bounds,
    pub dirty_rect: Bounds,
    pub root_id: String,
    focused: Option<String>,
}

impl<C, F> Scene<C, F> {
    pub(crate) fn get_children(&self, name: &str) -> Vec<String> {
        if let Some(children) = self.children.get(name) {
            children.clone()
        } else {
            Vec::new()
        }
    }
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
    pub fn is_focused(&self, name: &str) -> bool {
        self.focused.as_ref().is_some_and(|focused| focused == name)
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
}

pub type Callback<C, F> = fn(event: &mut GuiEvent<C, F>);

#[derive(Debug)]
pub enum EventType {
    Generic,
    Tap(Point),
    Scroll(i32, i32),
    Keyboard(u8),
    Action(),
}
#[derive(Debug)]
pub struct GuiEvent<'a, C, F> {
    pub scene: &'a mut Scene<C, F>,
    pub target: &'a str,
    pub event_type: EventType,
    pub action: Option<Action>,
}

#[derive(Debug)]
pub struct LayoutEvent<'a, C, F> {
    pub scene: &'a mut Scene<C, F>,
    pub target: &'a str,
}

impl<C, F> Scene<C, F> {
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
            draw: Some(draw_root_view),
            input: None,
            state: None,
            layout: None,
            draw2: None,
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

// pub fn connect_parent_child<C, F>(scene: &mut Scene<C, F>, parent: &str, child: &str) {
//     if !scene.children.contains_key(parent) {
//         scene.children.insert(parent.to_string(), vec![]);
//     }
//     if let Some(children) = scene.children.get_mut(parent) {
//         children.push(child.to_string());
//     }
// }
pub fn remove_parent_child<C, F>(scene: &mut Scene<C, F>, parent: &str, child: &str) {
    if let Some(children) = scene.children.get_mut(parent) {
        if let Some(n) = children.iter().position(|name| name == child) {
            children.remove(n);
        }
    }
}
pub fn click_at<C, F>(scene: &mut Scene<C, F>, handlers: &Vec<Callback<C, F>>, pt: Point) {
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
    }
}
pub fn type_at_focused<C, F>(scene: &mut Scene<C, F>, handlers: &Vec<Callback<C, F>>, key: u8) {
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
        }
    }
}

pub fn scroll_at_focused<C, F>(
    scene: &mut Scene<C, F>,
    handlers: &Vec<Callback<C, F>>,
    dx: i32,
    dy: i32,
) {
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
        }
    }
}

pub fn action_at_focused<C, F>(scene: &mut Scene<C, F>, handlers: &Vec<Callback<C, F>>) {
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
        }
    }
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
pub fn layout_vbox<C, F>(evt: &mut LayoutEvent<C, F>) {
    if let Some(parent) = evt.scene.get_view_mut(evt.target) {
        let mut y = 0;
        let bounds = parent.bounds;
        let kids = evt.scene.get_children(evt.target);
        for kid in kids {
            if let Some(ch) = evt.scene.get_view_mut(&kid) {
                ch.bounds.x = 0;
                ch.bounds.y = y;
                ch.bounds.w = bounds.w;
                y += ch.bounds.h;
            }
        }
    }
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

pub fn draw_view<C, F>(
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

pub fn layout_view<C, F>(scene: &mut Scene<C, F>, name: &str) {
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

fn draw_root_view<C, F>(
    view: &mut View<C, F>,
    ctx: &mut dyn DrawingContext<C, F>,
    theme: &Theme<C, F>,
) {
    ctx.fill_rect(&view.bounds, &theme.panel_bg)
}
// pub fn draw_button_view<C, F>(
//     view: &View<C, F>,
//     ctx: &mut dyn DrawingContext<C, F>,
//     theme: &Theme<C, F>,
// ) {
//     ctx.fill_rect(&view.bounds, &theme.bg);
//     ctx.stroke_rect(&view.bounds, &theme.fg);
//     ctx.fill_text(&view.bounds, &view.title, &theme.fg, &HAlign::Center);
// }
pub fn draw_panel_view<C, F>(
    view: &mut View<C, F>,
    ctx: &mut dyn DrawingContext<C, F>,
    theme: &Theme<C, F>,
) {
    ctx.fill_rect(&view.bounds, &theme.panel_bg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comps::make_button;
    use log::LevelFilter;
    use std::sync::Once;

    extern crate std;

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            // env_logger::Builder::new()
            //     // .format(|f, record| {
            //     //     writeln!(f,"[{}] - {}",record.level(),record.args())
            //     // })
            //     .target(env_logger::Target::Stdout) // <-- redirects to stdout
            //     .filter(None, LevelFilter::Info)
            //     .init();
        });
    }
    fn draw_generic_view<C, F>(
        view: &mut View<C, F>,
        ctx: &mut dyn DrawingContext<C, F>,
        theme: &Theme<C, F>,
    ) {
        ctx.fill_rect(&view.bounds, &theme.bg)
    }
    fn make_simple_view<C, F>(name: &str) -> View<C, F> {
        View {
            name: name.to_string(),
            title: name.to_string(),
            bounds: Bounds {
                x: 0,
                y: 0,
                w: 10,
                h: 10,
            },
            visible: true,
            draw: Some(draw_generic_view),
            draw2: None,
            input: None,
            state: None,
            layout: None,
        }
    }
    fn make_vbox<C, F>(name: &str, bounds: Bounds) -> View<C, F> {
        View {
            name: name.to_string(),
            title: name.to_string(),
            bounds,
            visible: true,
            draw: Some(draw_panel_view),
            draw2: None,
            input: None,
            state: None,
            layout: Some(layout_vbox),
        }
    }
    struct TestButtonState {
        drawn: bool,
        got_input: bool,
    }
    fn make_test_button<C, F>(name: &str) -> View<C, F> {
        View {
            name: name.to_string(),
            title: name.to_string(),
            bounds: Bounds {
                x: 0,
                y: 0,
                w: 20,
                h: 20,
            },
            visible: true,
            draw: Some(|view, _ctx, _theme| {
                if let Some(state) = &mut view.state {
                    if let Some(state) = state.downcast_mut::<TestButtonState>() {
                        state.drawn = true;
                    }
                }
            }),
            draw2: None,
            input: Some(|e| {
                if let Some(view) = e.scene.get_view_mut(e.target) {
                    if let Some(state) = &mut view.state {
                        if let Some(state) = state.downcast_mut::<TestButtonState>() {
                            state.got_input = true;
                        }
                    }
                }
                None
            }),
            state: Some(Box::new(TestButtonState {
                drawn: false,
                got_input: false,
            })),
            layout: None,
        }
    }
    fn make_text_box<C, F>(name: &str, title: &str) -> View<C, F> {
        View {
            name: name.into(),
            title: title.into(),
            bounds: Bounds::new(0, 0, 100, 30),
            visible: true,
            state: None,
            draw: None,
            draw2: None,
            layout: None,
            input: Some(|e| {
                match e.event_type {
                    EventType::Keyboard(key) => {
                        info!("got a keyboard event {}", key);
                        if let Some(view) = e.scene.get_view_mut(e.target) {
                            view.title.push(key as char)
                        }
                    }
                    _ => info!("ignoring other event"),
                };
                None
            }),
        }
    }
    fn draw_label_view<C, F>(
        view: &mut View<C, F>,
        ctx: &mut dyn DrawingContext<C, F>,
        theme: &Theme<C, F>,
    ) {
        ctx.fill_text(&view.bounds, &view.title, &TextStyle::new(&theme.font, &theme.fg));
    }
    fn make_label<C, F>(name: &str) -> View<C, F> {
        View {
            name: name.to_string(),
            title: name.to_string(),
            bounds: Bounds {
                x: 0,
                y: 0,
                w: 30,
                h: 20,
            },
            visible: true,
            draw: Some(draw_label_view),
            draw2: None,
            input: None,
            state: None,
            layout: None,
        }
    }
    fn get_bounds<C, F>(scene: &Scene<C, F>, name: &str) -> Option<Bounds> {
        if let Some(view) = scene.keys.get(name) {
            Some(view.bounds)
        } else {
            None
        }
    }
    fn was_button_clicked<C, F>(scene: &mut Scene<C, F>, name: &str) -> bool {
        scene
            .get_view(name)
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .downcast_ref::<TestButtonState>()
            .unwrap()
            .got_input
    }
    fn was_button_drawn<C, F>(scene: &mut Scene<C, F>, name: &str) -> bool {
        scene
            .get_view(name)
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .downcast_ref::<TestButtonState>()
            .unwrap()
            .drawn
    }

    fn repaint(scene: &mut Scene<String, String>) {
        let theme: Theme<String, String> = Theme {
            bg: "white".into(),
            fg: "black".into(),
            panel_bg: "grey".into(),
            font: "plain".into(),
            bold_font: "bold".into(),
        };

        let mut ctx: MockDrawingContext<String, String> = MockDrawingContext {
            bg: String::new(),
            font: String::new(),
            clip: scene.dirty_rect,
        };
        draw_scene(scene, &mut ctx, &theme);
        scene.dirty_rect = Bounds::new_empty();
    }

    #[test]
    fn test_geometry() {
        initialize();
        let bounds = Bounds {
            x: 0,
            y: 0,
            w: 100,
            h: 100,
        };
        assert_eq!(bounds.contains(&Point::new(10, 10)), true);
        assert_eq!(bounds.contains(&Point::new(-1, -1)), false);

        let b2 = Bounds::new(140, 180, 80, 30);
        let b3 = Bounds::new(140, 180, 80, 30);
        // INFO - union Bounds { x: 140, y: 180, w: 80, h: 30 } Bounds { x: 140, y: 180, w: 80, h: 30 }
        assert_eq!(b2.union(b3), b2.clone());
    }
    #[test]
    fn basic_add_remove() {
        let mut scene: Scene<String, String> = Scene::new_with_bounds(Bounds::new(0, 0, 100, 30));
        assert_eq!(scene.viewcount(), 1);
        let view = make_simple_view("foo");
        assert_eq!(scene.viewcount(), 1);
        scene.add_view(view);
        assert_eq!(scene.viewcount(), 2);
        assert!(scene.get_view("foo").is_some());
        let res = scene.remove_view("foo");
        assert_eq!(res.is_some(), true);
        assert_eq!(scene.viewcount(), 1);
        let res2 = scene.remove_view("bar");
        assert_eq!(res2.is_some(), false);
    }
    #[test]
    fn parent_child() {
        let mut scene: Scene<String, String> = Scene::new();
        scene.add_view(make_simple_view("parent"));
        scene.add_view(make_simple_view("child"));
        assert_eq!(scene.get_children("parent").len(), 0);
        assert_eq!(scene.viewcount(), 3);
        scene.add_child("parent", "child");
        assert_eq!(scene.get_children("parent").len(), 1);
        scene.remove_child("parent", "child");
        assert_eq!(scene.get_children("parent").len(), 0);

        scene.add_child("parent", "child");
        assert_eq!(scene.get_children("parent").len(), 1);
        let child2 = make_simple_view("child2");
        scene.add_view_to_parent(child2, "parent");
        assert_eq!(scene.get_children("parent").len(), 2);
        assert_eq!(scene.viewcount(), 4);

        scene.remove_parent_and_children("parent");
        assert_eq!(scene.get_children("parent").len(), 0);
        assert_eq!(scene.viewcount(), 1);
    }
    #[test]
    fn test_pick_at() {
        initialize();
        let mut scene: Scene<String, String> = Scene::new();
        let vbox = make_vbox(
            "parent",
            Bounds {
                x: 10,
                y: 10,
                w: 100,
                h: 100,
            },
        );

        let mut button = make_test_button("child");
        button.bounds = Bounds {
            x: 10,
            y: 10,
            w: 10,
            h: 10,
        };

        scene.add_child(&scene.root_id.clone(), &vbox.name);
        scene.add_child(&vbox.name, &button.name);
        scene.add_view(vbox);
        scene.add_view(button);
        assert_eq!(pick_at(&mut scene, &Point { x: 5, y: 5 }).len(), 1);
        assert_eq!(pick_at(&mut scene, &Point { x: 15, y: 15 }).len(), 2);
        assert_eq!(pick_at(&mut scene, &Point { x: 25, y: 25 }).len(), 3);
    }
    #[test]
    fn test_layout() {
        let mut scene: Scene<String, String> = Scene::new();
        // add panel
        scene.add_view(make_vbox(
            "parent",
            Bounds {
                x: 10,
                y: 10,
                w: 100,
                h: 100,
            },
        ));
        // add button 1
        scene.add_view_to_parent(make_test_button("button1"), "parent");
        // add button 2
        scene.add_view_to_parent(make_label("button2"), "parent");
        // layout
        layout_vbox(&mut LayoutEvent {
            scene: &mut scene,
            target: "parent",
        });
        assert_eq!(
            get_bounds(&scene, "parent"),
            Some(Bounds {
                x: 10,
                y: 10,
                w: 100,
                h: 100
            })
        );
        assert_eq!(
            get_bounds(&scene, "button1"),
            Some(Bounds {
                x: 0,
                y: 0,
                w: 100,
                h: 20
            })
        );
        assert_eq!(
            get_bounds(&scene, "button2"),
            Some(Bounds {
                x: 0,
                y: 20,
                w: 100,
                h: 20
            })
        );
    }
    #[test]
    fn test_repaint() {
        let mut scene: Scene<String, String> = Scene::new();
        // add panel
        scene.add_view(make_vbox(
            "parent",
            Bounds {
                x: 10,
                y: 10,
                w: 100,
                h: 100,
            },
        ));
        // add button 1
        scene.add_view(make_test_button("button1"));
        // add button 2
        scene.add_view(make_test_button("button2"));

        assert_eq!(scene.dirty, true);
        repaint(&mut scene);
        assert_eq!(scene.dirty, false);
    }
    #[test]
    fn test_events() {
        let mut scene: Scene<String, String> = Scene::new();
        let mut handlers: Vec<Callback<String, String>> = vec![];
        handlers.push(|event| {
            info!("got an event {:?}", event);
            if let Some(view) = event.scene.get_view_mut(event.target) {
                view.visible = false;
            }
            event.scene.dirty = true;
        });
        handlers.push(|event| {
            info!("got another event {:?}", event);
            if let Some(view) = event.scene.get_view_mut(event.target) {
                view.visible = false;
            }
            event.scene.dirty = true;
            info!("the action is {:?}", event.action);
        });
        assert_eq!(scene.get_view("root").unwrap().visible, true);
        click_at(&mut scene, &handlers, Point::new(5, 5));
        assert_eq!(scene.get_view("root").unwrap().visible, false);
    }
    fn handle_toggle_button_input<C, F>(event: &mut GuiEvent<C, F>) -> Option<Action> {
        // info!("view clicked {:?}", event.event_type);
        if let Some(view) = event.scene.get_view_mut(event.target) {
            view.state.insert(Box::new(String::from("enabled")));
        }
        None
    }
    #[test]
    fn test_toggle_button() {
        initialize();
        let mut scene = Scene::new();
        // add toggle button
        let button:View<String,String> = View {
            name: String::from("toggle"),
            title: String::from("Off"),
            visible: true,
            bounds: Bounds {
                x: 10,
                y: 10,
                w: 20,
                h: 20,
            },
            draw: Some(|view, ctx, theme| {
                if let Some(state) = &view.state {
                    if let Some(state) = state.downcast_ref::<String>() {
                        if state == "enabled" {
                            ctx.fill_rect(&view.bounds, &theme.fg);
                            ctx.stroke_rect(&view.bounds, &theme.bg);
                            let style = TextStyle::new(&theme.font, &theme.bg).with_halign(HAlign::Center);
                            ctx.fill_text(&view.bounds, &view.title, &style);
                        } else {
                            ctx.fill_rect(&view.bounds, &theme.bg);
                            ctx.stroke_rect(&view.bounds, &theme.fg);
                            let style = TextStyle::new(&theme.font, &theme.fg).with_halign(HAlign::Center);
                            ctx.fill_text(&view.bounds, &view.title, &style);
                        }
                    }
                }
            }),
            draw2: None,
            input: Some(handle_toggle_button_input),
            state: Some(Box::new(String::from("disabled"))),
            layout: None,
        };
        scene.add_view_to_root(button);
        // repaint
        repaint(&mut scene);
        assert_eq!(scene.get_view("toggle").unwrap().visible, true);
        assert_eq!(
            &scene
                .get_view("toggle")
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .downcast_ref::<String>()
                .unwrap(),
            &"disabled"
        );
        // click at
        let handlers: Vec<Callback<String, String>> = vec![];
        click_at(&mut scene, &handlers, Point::new(15, 15));
        // confirm toggle button state has changed to enabled
        assert_eq!(
            &scene
                .get_view("toggle")
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .downcast_ref::<String>()
                .unwrap(),
            &"enabled"
        );
    }
    #[test]
    fn test_make_visible() {
        // create scene
        initialize();
        let mut scene = Scene::new();

        // create button 1
        let mut button1 = make_test_button("button1");
        button1.visible = true;
        scene.add_view_to_root(button1);

        // create button 2
        let mut button2 = make_test_button("button2");
        button2.bounds.x = 100;
        // make button 2 invisible
        button2.visible = false;
        scene.add_view_to_root(button2);

        assert_eq!(was_button_clicked(&mut scene, "button1"), false);
        assert_eq!(was_button_drawn(&mut scene, "button1"), false);
        assert_eq!(was_button_drawn(&mut scene, "button2"), false);

        // repaint. only button 1 should get drawn
        repaint(&mut scene);
        assert_eq!(scene.dirty, false);
        assert_eq!(was_button_drawn(&mut scene, "button1"), true);
        assert_eq!(was_button_drawn(&mut scene, "button2"), false);

        let mut handlers: Vec<Callback<String, String>> = vec![];
        handlers.push(|e| {
            info!("clicked on {}", e.target);
            if let Some(view) = e.scene.get_view_mut("button2") {
                view.visible = true;
                e.scene.dirty = true;
            }
        });

        // tap button 1
        assert_eq!(scene.dirty, false);
        click_at(&mut scene, &handlers, Point::new(15, 15));
        assert_eq!(was_button_clicked(&mut scene, "button1"), true);
        // confirm dirty
        assert_eq!(scene.dirty, true);

        // this time both buttons should be drawn
        repaint(&mut scene);
        assert_eq!(scene.dirty, false);
        assert_eq!(was_button_drawn(&mut scene, "button1"), true);
        assert_eq!(was_button_drawn(&mut scene, "button2"), true);
    }
    #[test]
    fn test_keyboard_evnts() {
        // make scene
        initialize();
        let mut scene = Scene::new();
        let rootid = scene.root_id.clone();

        // make text box
        let text_box = make_text_box("textbox1", "foo");
        scene.add_view_to_root(text_box);
        // confirm text is correct
        assert_eq!(get_view_title(&scene, "textbox1"), "foo");
        // set text box as focused
        scene.focused = Some("textbox1".into());

        // send keyboard event
        let handlers: Vec<Callback<String, String>> = vec![];
        type_at_focused(&mut scene, &handlers, b'X');
        // confirm text is updated
        assert_eq!(get_view_title(&scene, "textbox1"), "fooX");
    }

    #[test]
    fn test_draw2() {
        initialize();
        let mut scene = Scene::new();
        let view: View<String, String> = View {
            name: "view".into(),
            title: "view".into(),
            bounds: Bounds::new(0, 0, 10, 10),
            visible: true,
            draw2: Some(|e| {
                let mut color = &e.theme.fg;
                if e.focused.is_some() && e.view.name.eq(e.focused.as_ref().unwrap()) {
                    color = &e.theme.bg;
                }
                e.ctx.fill_rect(&e.view.bounds, color);
            }),
            state: None,
            draw: None,
            input: None,
            layout: None,
        };

        scene.add_view_to_root(view);
        repaint(&mut scene);
    }

    #[test]
    fn test_cliprect() {
        initialize();
        // make scene
        let mut scene: Scene<String, String> = Scene::new();
        // add button
        let button = make_button("button", "Button").position_at(20, 20);
        scene.add_view_to_root(button);
        assert_eq!(scene.dirty, true);
        // check that dirty area is same as bounds
        assert_eq!(scene.dirty_rect, scene.bounds);
        assert_eq!(scene.dirty_rect.is_empty(), false);
        // draw
        repaint(&mut scene);
        // check that dirty area is empty
        assert_eq!(scene.dirty, false);
        assert_eq!(scene.dirty_rect.is_empty(), true);
        // send tap to button
        let handlers: Vec<Callback<String, String>> = vec![];
        click_at(&mut scene, &handlers, Point::new(30, 30));
        // check that dirty area is just for the button
        assert_eq!(scene.dirty, true);
        assert_eq!(scene.dirty_rect, scene.get_view("button").unwrap().bounds);
        // draw
        repaint(&mut scene);
        assert_eq!(scene.dirty, false);
        assert_eq!(scene.dirty_rect.is_empty(), true);
        // check that button was redrawn
    }

    fn get_view_title<C, F>(scene: &Scene<C, F>, name: &str) -> String {
        scene.get_view(name).unwrap().title.clone()
    }
}

pub struct MockDrawingContext<C, F> {
    pub clip: Bounds,
    pub bg: C,
    pub font: F,
}
impl DrawingContext<String, String> for MockDrawingContext<String, String> {
    fn clear(&mut self, _color: &String) {}

    fn fill_rect(&mut self, _bounds: &Bounds, _color: &String) {}

    fn stroke_rect(&mut self, _bounds: &Bounds, _color: &String) {}

    fn fill_text(&mut self, _bounds: &Bounds, _text: &str, _style:&TextStyle<String,String>) {}
}
