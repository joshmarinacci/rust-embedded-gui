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

pub mod geom;

pub trait DrawingContext<C> {
    fn clear(&mut self, color: &C);
    fn fillRect(&mut self, bounds: &Bounds, color: &C);
    fn strokeRect(&mut self, bounds: &Bounds, color: &C);
    fn fillText(&mut self, bounds: &Bounds, text: &str, color: &C);
}

pub type DrawFn<C> = fn(view: &mut View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>);
pub type LayoutFn<C> = fn(scene: &mut Scene<C>, name: &str);
pub type InputFn<C> = fn(event: &mut GuiEvent<C>);

pub struct Theme<C> {
    pub bg: C,
    pub fg: C,
    pub panel_bg: C,
}

#[derive(Debug)]
pub struct View<C> {
    pub name: String,
    pub title: String,
    pub bounds: Bounds,
    pub visible: bool,
    pub draw: Option<DrawFn<C>>,
    pub input: Option<InputFn<C>>,
    pub state: Option<Box<dyn Any>>,
    pub layout: Option<LayoutFn<C>>,
    pub children: Vec<String>,
}

#[derive(Debug)]
pub struct Scene<C> {
    keys: HashMap<String, View<C>>,
    pub dirty: bool,
    bounds: Bounds,
    pub rootId: String,
    pub focused: Option<String>,
}

impl<C> Scene<C> {
    pub fn set_focused(&mut self, name: &str) {
        self.focused = Some(name.into());
        self.dirty = true
    }
}

pub type Callback<C> = fn(event: &mut GuiEvent<C>);

#[derive(Debug)]
pub enum EventType {
    Generic,
    Tap(Point),
    Scroll(i32, i32),
    Keyboard(u8),
}
#[derive(Debug)]
pub struct GuiEvent<'a, C> {
    pub scene: &'a mut Scene<C>,
    pub target: &'a str,
    pub event_type: EventType,
}

impl<C> Scene<C> {
    pub(crate) fn has_view(&self, name: &str) -> bool {
        self.keys.get(name).is_some()
    }
    pub fn get_view(&self, name: &str) -> Option<&View<C>> {
        self.keys.get(name)
    }
    pub fn get_view_mut(&mut self, name: &str) -> Option<&mut View<C>> {
        self.keys.get_mut(name)
    }
    pub(crate) fn viewcount(&self) -> usize {
        self.keys.len()
    }
    pub fn remove_view(&mut self, name: &str) -> Option<View<C>> {
        self.keys.remove(name)
    }
    pub fn new() -> Scene<C> {
        let bounds = Bounds {
            x: 0,
            y: 0,
            w: 200,
            h: 200,
        };
        let root = View {
            name: "root".to_string(),
            title: "root".to_string(),
            bounds: bounds.clone(),
            visible: true,
            draw: Some(draw_root_view),
            input: None,
            state: None,
            layout: None,
            children: vec![],
        };
        let rootId = String::from("root");
        let mut keys: HashMap<String, View<C>> = HashMap::new();
        keys.insert(rootId.clone(), root);
        Scene {
            bounds,
            keys,
            dirty: true,
            rootId,
            focused: None,
        }
    }
    pub fn add_view(&mut self, view: View<C>) {
        self.keys.insert(view.name.clone(), view);
    }
}

pub fn connect_parent_child<C>(scene: &mut Scene<C>, parent: &str, child: &str) {
    if let Some(view) = scene.get_view_mut(parent) {
        view.children.push(child.into());
    }
}
pub fn remove_parent_child<C>(scene: &mut Scene<C>, parent: &str, child: &str) {
    if let Some(view) = scene.get_view_mut(parent) {
        if let Some(n) = view.children.iter().position(|name| name == child) {
            view.children.remove(n);
        }
    }
}
pub fn click_at<C>(scene: &mut Scene<C>, handlers: &Vec<Callback<C>>, pt: Point) {
    // info!("picking at {:?}", pt);
    let targets = pick_at(scene, &pt);
    if let Some(target) = targets.last() {
        // info!("doing the target {}", target);
        let mut event: GuiEvent<C> = GuiEvent {
            scene: scene,
            target: target,
            event_type: EventType::Tap(pt),
        };
        if let Some(view) = event.scene.get_view(target) {
            // info!("got the view {:?}", view.name);
            if let Some(input) = view.input {
                input(&mut event)
            }
        }
        for cb in handlers {
            cb(&mut event);
        }
    }
}
pub fn type_at_focused<C>(scene: &mut Scene<C>, handlers: &Vec<Callback<C>>, key: u8) {
    if scene.focused.is_none() {
        return;
    } else {
        let focused = scene.focused.as_ref().unwrap().clone();
        let mut event: GuiEvent<C> = GuiEvent {
            scene: scene,
            target: &focused,
            event_type: EventType::Keyboard(key),
        };
        if let Some(view) = event.scene.get_view(&focused) {
            if let Some(input) = view.input {
                input(&mut event)
            }
            for cb in handlers {
                cb(&mut event);
            }
        }
    }
}
pub fn pick_at<C>(scene: &mut Scene<C>, pt: &Point) -> Vec<String> {
    pick_at_view(scene, pt, &scene.rootId)
}
fn pick_at_view<C>(scene: &Scene<C>, pt: &Point, name: &str) -> Vec<String> {
    let mut coll: Vec<String> = vec![];
    if let Some(view) = scene.keys.get(name) {
        if view.bounds.contains(pt) {
            coll.push(view.name.clone());
            let pt2 = Point {
                x: pt.x - view.bounds.x,
                y: pt.y - view.bounds.y,
            };
            for kid in find_children(scene, &view.name) {
                let mut coll2 = pick_at_view(scene, &pt2, &kid);
                coll.append(&mut coll2);
            }
        }
    }
    coll
}
pub fn find_children<C>(scene: &Scene<C>, parent: &str) -> Vec<String> {
    if let Some(view) = scene.get_view(parent) {
        view.children.clone()
    } else {
        vec![]
    }
}

pub fn layout_vbox<C>(scene: &mut Scene<C>, name: &str) {
    let parent = scene.keys.get_mut(name);
    if let Some(parent) = parent {
        let mut y = 0;
        let bounds = parent.bounds.clone();
        let kids = find_children(scene, name);
        for kid in kids {
            if let Some(ch) = scene.keys.get_mut(&kid) {
                ch.bounds.x = 0;
                ch.bounds.y = y;
                ch.bounds.w = bounds.w;
                y += ch.bounds.h;
            }
        }
    }
}
fn get_child_count<C>(scene: &mut Scene<C>, name: &str) -> usize {
    if let Some(view) = scene.get_view(name) {
        view.children.len()
    } else {
        0
    }
}

fn repaint(scene: &mut Scene<String>) {
    let theme: Theme<String> = Theme {
        bg: "white".into(),
        fg: "black".into(),
        panel_bg: "grey".into(),
    };

    let mut ctx: FakeDrawingContext<String> = FakeDrawingContext {
        clip: Bounds {
            x: 0,
            y: 0,
            w: 200,
            h: 200,
        },
        bg: String::new(),
    };
    draw_scene(scene, &mut ctx, &theme);
}

pub fn draw_scene<C>(scene: &mut Scene<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    if scene.dirty {
        ctx.clear(&theme.panel_bg);
        let name = scene.rootId.clone();
        draw_view(scene, ctx, theme, &name);
        scene.dirty = false;
    }
}

pub fn draw_view<C>(
    scene: &mut Scene<C>,
    ctx: &mut dyn DrawingContext<C>,
    theme: &Theme<C>,
    name: &str,
) {
    if let Some(view) = scene.get_view_mut(name) {
        if view.visible {
            (view.draw.unwrap())(view, ctx, &theme);
        }
    }
    if let Some(view) = scene.get_view(name) {
        for kid in find_children(&scene, &view.name) {
            draw_view(scene, ctx, theme, &kid);
        }
    }
}

fn draw_generic_view<C>(view: &mut View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    ctx.fillRect(&view.bounds, &theme.bg)
}
fn draw_root_view<C>(view: &mut View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    ctx.fillRect(&view.bounds, &theme.panel_bg)
}
pub fn draw_button_view<C>(view: &View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    ctx.fillRect(&view.bounds, &theme.bg);
    ctx.strokeRect(&view.bounds, &theme.fg);
    ctx.fillText(&view.bounds, &view.title, &theme.fg);
}
fn draw_toggle_button_view<C>(
    view: &mut View<C>,
    ctx: &mut dyn DrawingContext<C>,
    theme: &Theme<C>,
) {
    if let Some(state) = &view.state {
        if let Some(state) = state.downcast_ref::<String>() {
            if state == "enabled" {
                ctx.fillRect(&view.bounds, &theme.fg);
                ctx.strokeRect(&view.bounds, &theme.bg);
                ctx.fillText(&view.bounds, &view.title, &theme.bg);
            } else {
                ctx.fillRect(&view.bounds, &theme.bg);
                ctx.strokeRect(&view.bounds, &theme.fg);
                ctx.fillText(&view.bounds, &view.title, &theme.fg);
            }
        }
    }
}
fn draw_label_view<C>(view: &mut View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    ctx.fillText(&view.bounds, &view.title, &theme.fg);
}
pub fn draw_panel_view<C>(view: &mut View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    ctx.fillRect(&view.bounds, &theme.panel_bg);
}

fn handle_toggle_button_input<C>(event: &mut GuiEvent<C>) {
    // info!("view clicked {:?}", event.event_type);
    if let Some(view) = event.scene.get_view_mut(event.target) {
        view.state.insert(Box::new(String::from("enabled")));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;
    use std::sync::Once;
    extern crate std;

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            env_logger::Builder::new()
                // .format(|f, record| {
                //     writeln!(f,"[{}] - {}",record.level(),record.args())
                // })
                .target(env_logger::Target::Stdout) // <-- redirects to stdout
                .filter(None, LevelFilter::Info)
                .init();
        });
    }
    fn make_simple_view<C>(name: &str) -> View<C> {
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
            children: vec![],
            draw: Some(draw_generic_view),
            input: None,
            state: None,
            layout: None,
        }
    }
    fn make_vbox<C>(name: &str, bounds: Bounds) -> View<C> {
        View {
            name: name.to_string(),
            title: name.to_string(),
            bounds,
            visible: true,
            children: vec![],
            draw: Some(draw_panel_view),
            input: None,
            state: None,
            layout: Some(layout_vbox),
        }
    }
    struct TestButtonState {
        drawn: bool,
        got_input: bool,
    }
    fn make_test_button<C>(name: &str) -> View<C> {
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
            children: vec![],
            draw: Some(|view, ctx, theme| {
                if let Some(state) = &mut view.state {
                    if let Some(state) = state.downcast_mut::<TestButtonState>() {
                        state.drawn = true;
                    }
                }
            }),
            input: Some(|e| {
                if let Some(view) = e.scene.get_view_mut(e.target) {
                    if let Some(state) = &mut view.state {
                        if let Some(state) = state.downcast_mut::<TestButtonState>() {
                            state.got_input = true;
                        }
                    }
                }
            }),
            state: Some(Box::new(TestButtonState {
                drawn: false,
                got_input: false,
            })),
            layout: None,
        }
    }
    fn make_text_box<C>(name: &str, title: &str) -> View<C> {
        View {
            name: name.into(),
            title: title.into(),
            bounds: Bounds::new(0, 0, 100, 30),
            visible: true,
            children: vec![],
            state: None,
            draw: None,
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
            }),
        }
    }
    fn make_label<C>(name: &str) -> View<C> {
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
            children: vec![],
            draw: Some(draw_label_view),
            input: None,
            state: None,
            layout: None,
        }
    }
    fn get_bounds<C>(scene: &Scene<C>, name: &str) -> Option<Bounds> {
        if let Some(view) = scene.keys.get(name) {
            Some(view.bounds)
        } else {
            None
        }
    }
    fn was_button_clicked<C>(scene: &mut Scene<C>, name: &str) -> bool {
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
    fn was_button_drawn<C>(scene: &mut Scene<C>, name: &str) -> bool {
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

    #[test]
    fn test_geometry() {
        let bounds = Bounds {
            x: 0,
            y: 0,
            w: 100,
            h: 100,
        };
        assert_eq!(bounds.contains(&Point::new(10, 10)), true);
        assert_eq!(bounds.contains(&Point::new(-1, -1)), false);
    }
    #[test]
    fn basic_add_remove() {
        let mut scene: Scene<String> = Scene::new();
        assert_eq!(scene.viewcount(), 1);
        let view: View<String> = make_simple_view("foo");
        assert_eq!(scene.viewcount(), 1);
        scene.add_view(view);
        assert_eq!(scene.viewcount(), 2);
        assert_eq!(scene.has_view("foo"), true);
        let res: Option<View<String>> = scene.remove_view("foo");
        assert_eq!(res.is_some(), true);
        assert_eq!(scene.viewcount(), 1);
        let res2: Option<View<String>> = scene.remove_view("bar");
        assert_eq!(res2.is_some(), false);
    }
    #[test]
    fn parent_child() {
        let mut scene: Scene<String> = Scene::new();
        scene.add_view(make_simple_view("parent"));
        scene.add_view(make_simple_view("child"));
        assert_eq!(scene.get_view("parent").unwrap().children.len(), 0);
        assert_eq!(scene.viewcount(), 3);
        connect_parent_child(&mut scene, "parent", "child");
        assert_eq!(scene.get_view("parent").unwrap().children.len(), 1);
        remove_parent_child(&mut scene, "parent", "child");
        assert_eq!(scene.get_view("parent").unwrap().children.len(), 0);
    }
    #[test]
    fn test_pick_at() {
        initialize();
        let mut scene: Scene<String> = Scene::new();
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
        // add button
        let mut button = make_test_button("child");
        button.bounds = Bounds {
            x: 10,
            y: 10,
            w: 10,
            h: 10,
        };
        scene.add_view(button);
        // connect
        connect_parent_child(&mut scene, "root", "parent");
        connect_parent_child(&mut scene, "parent", "child");
        assert_eq!(pick_at(&mut scene, &Point { x: 5, y: 5 }).len(), 1);
        assert_eq!(pick_at(&mut scene, &Point { x: 15, y: 15 }).len(), 2);
        assert_eq!(pick_at(&mut scene, &Point { x: 25, y: 25 }).len(), 3);
    }
    #[test]
    fn test_layout() {
        let mut scene: Scene<String> = Scene::new();
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
        scene.add_view(make_label("button2"));
        // connect
        connect_parent_child(&mut scene, "parent", "button1");
        connect_parent_child(&mut scene, "parent", "button2");
        // layout
        layout_vbox(&mut scene, "parent");
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
        let mut scene: Scene<String> = Scene::new();
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
        let mut scene: Scene<String> = Scene::new();
        let mut handlers: Vec<Callback<String>> = vec![];
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
        });
        assert_eq!(scene.get_view("root").unwrap().visible, true);
        click_at(&mut scene, &handlers, Point::new(5, 5));
        assert_eq!(scene.get_view("root").unwrap().visible, false);
    }
    #[test]
    fn test_toggle_button() {
        initialize();
        let mut scene = Scene::new();
        // add toggle button
        let button = View {
            name: String::from("toggle"),
            title: String::from("Off"),
            visible: true,
            bounds: Bounds {
                x: 10,
                y: 10,
                w: 20,
                h: 20,
            },
            children: vec![],
            draw: Some(draw_toggle_button_view),
            input: Some(handle_toggle_button_input),
            state: Some(Box::new(String::from("disabled"))),
            layout: None,
        };
        scene.add_view(button);
        connect_parent_child(&mut scene, "root", "toggle");
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
        let mut handlers: Vec<Callback<String>> = vec![];
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
        let rootid = scene.rootId.clone();

        // create button 1
        let mut button1 = make_test_button("button1");
        button1.visible = true;
        connect_parent_child(&mut scene, &rootid, &button1.name);
        scene.add_view(button1);

        // create button 2
        let mut button2 = make_test_button("button2");
        button2.bounds.x = 100;
        // make button 2 invisible
        button2.visible = false;
        connect_parent_child(&mut scene, &rootid, &button2.name);
        scene.add_view(button2);

        assert_eq!(was_button_clicked(&mut scene, "button1"), false);
        assert_eq!(was_button_drawn(&mut scene, "button1"), false);
        assert_eq!(was_button_drawn(&mut scene, "button2"), false);

        // repaint. only button 1 should get drawn
        repaint(&mut scene);
        assert_eq!(scene.dirty, false);
        assert_eq!(was_button_drawn(&mut scene, "button1"), true);
        assert_eq!(was_button_drawn(&mut scene, "button2"), false);

        let mut handlers: Vec<Callback<String>> = vec![];
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
        let rootid = scene.rootId.clone();

        // make text box
        let mut text_box = make_text_box("textbox1", "foo");
        connect_parent_child(&mut scene, &rootid, &text_box.name);
        scene.add_view(text_box);
        // confirm text is correct
        assert_eq!(get_view_title(&scene, "textbox1"), "foo");
        // set text box as focused
        scene.focused = Some("textbox1".into());

        // send keyboard event
        let mut handlers: Vec<Callback<String>> = vec![];
        type_at_focused(&mut scene, &handlers, b'X');
        // confirm text is updated
        assert_eq!(get_view_title(&scene, "textbox1"), "fooX");
    }

    fn get_view_title<C>(scene: &Scene<C>, name: &str) -> String {
        scene.get_view(name).unwrap().title.clone()
    }
}

struct FakeDrawingContext<C> {
    clip: Bounds,
    bg: C,
}
impl DrawingContext<String> for FakeDrawingContext<String> {
    fn clear(&mut self, color: &String) {}

    fn fillRect(&mut self, bounds: &Bounds, color: &String) {}

    fn strokeRect(&mut self, bounds: &Bounds, color: &String) {}

    fn fillText(&mut self, bounds: &Bounds, text: &str, color: &String) {}
}
