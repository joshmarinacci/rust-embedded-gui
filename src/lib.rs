#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate core;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;
use hashbrown::HashMap;
use log::info;
use geom::{Bounds, Point};

pub mod geom;

pub trait DrawingContext<C> {
    fn fillRect(&mut self, bounds: &Bounds, color: &C);
    fn strokeRect(&mut self, bounds: &Bounds, color: &C);
    fn fillText(&mut self, bounds: &Bounds, text: &str, color: &C);
}

pub type DrawFn<C> = fn(view: &View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>);
pub type LayoutFn<C> = fn(scene: &mut Scene<C>, name: &str);
pub type InputFn<C> = fn(view: &mut View<C>);

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
}

#[derive(Debug)]
pub struct Scene<C> {
    keys: HashMap<String, View<C>>,
    connections: Vec<Connection>,
    pub dirty: bool,
    bounds: Bounds,
    pub rootId: String,
    focused: Option<String>,
}

pub type Callback<C> = fn(event: &mut GuiEvent<C>);

#[derive(Debug)]
pub enum EventType {
    Generic,
    Tap(Point),
    Scroll(i32,i32),
    Keyboard(u8),
}
#[derive(Debug)]
pub struct GuiEvent<'a, C> {
    pub scene: &'a mut Scene<C>,
    pub target: &'a str,
    pub event_type: EventType,
}

impl<C> Scene<C> {
    pub(crate) fn connectioncount(&self) -> usize {
        self.connections.len()
    }
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
        };
        let rootId = String::from("root");
        let mut keys: HashMap<String, View<C>> = HashMap::new();
        keys.insert(rootId.clone(), root);
        Scene {
            bounds,
            keys,
            connections: vec![],
            dirty: true,
            rootId,
            focused: None,
        }
    }
    pub fn add_view(&mut self, view: View<C>) {
        self.keys.insert(view.name.clone(), view);
    }
}

#[derive(Debug)]
struct Connection {
    parent: String,
    child: String,
}
pub fn connect_parent_child<C>(scene: &mut Scene<C>, parent: &str, child: &str) {
    scene.connections.push(Connection {
        parent: parent.to_string(),
        child: child.to_string(),
    })
}
pub fn remove_parent_child<C>(scene: &mut Scene<C>, parent: &str, child: &str) -> Option<Connection> {
    if let Some(n) = scene
        .connections
        .iter()
        .position(|c| c.parent == parent && c.child == child)
    {
        return Some(scene.connections.remove(n));
    }
    None
}
pub fn click_at<C>(scene: &mut Scene<C>, handlers: &Vec<Callback<C>>, pt: Point) {
    // info!("picking at {:?}", pt);
    let targets = pick_at(scene, &pt);
    if let Some(target) = targets.last() {
        // info!("doing the target {}", target);
        if let Some(view) = scene.get_view_mut(target) {
            // info!("got the view {:?}", view);
            if let Some(input) = view.input {
                input(view)
            }
        }
        let mut event: GuiEvent<C> = GuiEvent {
            scene: scene,
            target: target,
            event_type: EventType::Generic,
        };
        for cb in handlers {
            cb(&mut event);
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
    let mut out = vec![];
    for con in &scene.connections {
        if con.parent == parent {
            out.push(con.child.clone());
        }
    }
    out
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
    let conn: Vec<&Connection> = scene
        .connections
        .iter()
        .filter(|c| c.parent == name)
        .collect();
    conn.len()
}

fn repaint(scene: &mut Scene<String>) {
    let theme:Theme<String> = Theme {
        bg: "white".into(),
        fg: "black".into(),
        panel_bg: "grey".into(),
    };

    let mut ctx:FakeDrawingContext<String> = FakeDrawingContext {
        clip: Bounds {
            x: 0,
            y: 0,
            w: 200,
            h: 200,
        },
        bg:String::new(),
    };
    if let Some(root) = scene.get_view(&scene.rootId) {
        (root.draw.unwrap())(root, &mut ctx, &theme);
        let kids = find_children(scene, &root.name);
        for kid in kids {
            if let Some(kid) = scene.get_view(&kid) {
                (kid.draw.unwrap())(root, &mut ctx, &theme);
            }
        }
        scene.dirty = false;
    }
}
fn draw_generic_view<C>(view: &View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    ctx.fillRect(&view.bounds, &theme.bg)
}
fn draw_root_view<C>(view: &View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    ctx.fillRect(&view.bounds, &theme.panel_bg)
}
pub fn draw_button_view<C>(view: &View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    ctx.fillRect(&view.bounds, &theme.bg);
    ctx.strokeRect(&view.bounds, &theme.fg);
    ctx.fillText(&view.bounds, &view.title, &theme.fg);
}
fn draw_toggle_button_view<C>(view: &View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
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
fn draw_label_view<C>(view: &View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    ctx.fillText(&view.bounds, &view.title, &theme.fg);
}
pub fn draw_panel_view<C>(view: &View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    ctx.fillRect(&view.bounds, &theme.panel_bg);
}

fn handle_toggle_button_input<C>(view: &mut View<C>) {
    // info!("view clicked {:?}", view);
    view.state.insert(Box::new(String::from("enabled")));
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
            draw: Some(draw_panel_view),
            input: None,
            state: None,
            layout: Some(layout_vbox),
        }
    }
    fn make_button<C>(name: &str) -> View<C> {
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
            draw: Some(draw_button_view),
            input: None,
            state: None,
            layout: None,
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
        assert_eq!(scene.connectioncount(), 0);
        assert_eq!(get_child_count(&mut scene, "parent"), 0);
        assert_eq!(scene.viewcount(), 3);

        connect_parent_child(&mut scene, "parent", "child");
        assert_eq!(scene.connectioncount(), 1);
        assert_eq!(get_child_count(&mut scene, "parent"), 1);

        remove_parent_child(&mut scene, "parent", "child");
        assert_eq!(scene.connectioncount(), 0);
        assert_eq!(get_child_count(&mut scene, "parent"), 0);
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
        let mut button = make_button("child");
        button.bounds = Bounds { x: 10, y: 10, w: 10, h:10};
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
        scene.add_view(make_button("button1"));
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
        scene.add_view(make_button("button1"));
        // add button 2
        scene.add_view(make_button("button2"));

        assert_eq!(scene.dirty, true);
        repaint(&mut scene);
        assert_eq!(scene.dirty, false);
    }
    #[test]
    fn test_events() {
        let mut scene:Scene<String> = Scene::new();
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
}

struct FakeDrawingContext<C> {
    clip: Bounds,
    bg:C,
}
impl DrawingContext<String> for FakeDrawingContext<String> {
    fn fillRect(&mut self, bounds: &Bounds, color: &String) {}

    fn strokeRect(&mut self, bounds: &Bounds, color: &String) {}

    fn fillText(&mut self, bounds: &Bounds, text: &str, color: &String) {}
}
