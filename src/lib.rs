#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate core;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use hashbrown::HashMap;
use log::info;

#[derive(Debug, PartialEq, Copy, Clone)]
struct Bounds {
    x: i32,
    y: i32,
    w: i32,
    h: i32
}

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

trait DrawingContext {
    fn fillRect(&self, bounds: &Bounds, color: &str);
    fn strokeRect(&self, bounds: &Bounds, color: &str);
    fn fillText(&self, bounds: &Bounds, text: &str, color:&str);
}

type DrawFn = fn(view:&View, ctx:&dyn DrawingContext, theme:&Theme);

#[derive(Debug)]
pub struct View {
    name:String,
    title:String,
    bounds: Bounds,
    visible: bool,
    draw:DrawFn,
}

#[derive(Debug)]
pub struct Scene {
    keys: HashMap<String, View>,
    connections: Vec<Connection>,
    dirty: bool,
    bounds: Bounds,
    rootId:String,
    focused:Option<String>,
}

type Callback = fn(event:&mut GuiEvent);

#[derive(Debug)]
struct GuiEvent<'a> {
    scene:&'a mut Scene,
    target: &'a str,
}

impl Scene {
    pub(crate) fn connectioncount(&self) -> usize {
        self.connections.len()
    }
    pub(crate) fn has_view(&self, name: &str) -> bool {
        self.keys.get(name).is_some()
    }
    pub fn get_view(&self, name: &str) -> Option<&View> {
        self.keys.get(name)
    }
    pub fn get_view_mut(&mut self, name: &str) -> Option<&mut View> {
        self.keys.get_mut(name)
    }
    pub(crate) fn viewcount(&self) -> usize {
        self.keys.len()
    }
    pub fn remove_view(&mut self, name: &str) -> Option<View> {
        self.keys.remove(name)
    }
    pub(crate) fn new() -> Scene {
        let bounds = Bounds {
            x:0,y:0, w:200,h:200,
        };
        let root = View {
            name:"root".to_string(),
            title:"root".to_string(),
            bounds: bounds.clone(),
            visible:true,
            draw: draw_root_view,
        };
        let rootId =String::from("root");
        let mut keys:HashMap<String, View> = HashMap::new();
        keys.insert(rootId.clone(),root);
        Scene {
            bounds,
            keys,
            connections: vec![],
            dirty:true,
            rootId,
            focused:None
        }
    }
    pub fn add_view(&mut self, view: View) {
        self.keys.insert(view.name.clone(),view);
    }
}

impl Point {
    pub(crate) fn new(x: i32, y: i32) -> Point {
        Point { x, y, }
    }
}


#[derive(Debug)]
struct Connection {
    parent: String,
    child: String,
}
fn connect_parent_child(scene: &mut Scene, parent: &str, child: &str) {
    scene.connections.push(Connection{parent:parent.to_string(), child:child.to_string()})
}
fn remove_parent_child(scene: &mut Scene, parent: &str, child: &str) -> Option<Connection> {
    if let Some(n) = scene.connections.iter().position(|c| c.parent == parent && c.child == child) {
        return Some(scene.connections.remove(n));
    }
    None
}
fn click_at(scene: &mut Scene, handlers:&Vec<Callback>, pt: Point) {
    let targets = pick_at(scene, &pt);
    if let Some(target) = targets.last() {
        let mut event:GuiEvent = GuiEvent {
            scene:scene,
            target:target
        };
        for cb in handlers {
            cb(&mut event);
        }
    }
}
fn pick_at(scene: &mut Scene, pt: &Point) -> Vec<String> {
    pick_at_view(scene, pt, &scene.rootId)
}
fn pick_at_view(scene: &Scene, pt: &Point, name:&str) -> Vec<String> {
    let mut coll:Vec<String> = vec![];
    if let Some(view) = scene.keys.get(name) {
        if view.bounds.contains(pt) {
            coll.push(view.name.clone());
            let pt2 = Point {
                x:pt.x- view.bounds.x,
                y:pt.y- view.bounds.y,
            };
            for kid in find_children(scene, &view.name) {
                let mut coll2 = pick_at_view(scene, &pt2, &kid);
                coll.append(&mut coll2);
            }
        }
    }
    coll
}
fn find_children(scene: &Scene, parent: &str) -> Vec<String> {
    let mut out = vec![];
    for con in &scene.connections {
        if con.parent == parent {
            out.push(con.child.clone());
        }
    }
    out
}

impl Bounds {
    pub(crate) fn contains(&self, pt: &Point) -> bool {
        if self.x <= pt.x && self.y <= pt.y {
            if self.x + self.w > pt.x && self.y + self.h > pt.y {
                return true;
            }
        }
        false
    }
}

struct Theme<'a> {
    bg:&'a str,
    fg:&'a str,
    panel_bg:&'a str,
}


fn get_bounds(scene: &Scene, name: &str) -> Option<Bounds> {
    if let Some(view) = scene.keys.get(name) {
        Some(view.bounds)
    } else {
        None
    }
}

fn layout_vbox(scene: &mut Scene, name: &str) {
    let parent = scene.keys.get_mut(name);
    if let Some(parent) = parent {
        let mut y = 0;
        let bounds = parent.bounds.clone();
        let kids = find_children(scene,name);
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
fn get_child_count(scene: &mut Scene, name: &str) -> usize {
    let conn:Vec<&Connection> = scene.connections.iter().filter(|c|c.parent == name).collect();
    conn.len()
}

fn repaint(scene: &mut Scene) {
    let theme = Theme {
        bg:"white",
        fg:"black",
        panel_bg:"grey"
    };

    let ctx = FakeDrawingContext{ clip: Bounds {x:0, y:0, w:200, h:200} };
    if let Some(root) = scene.get_view(&scene.rootId) {
        (root.draw)(root,&ctx,&theme);
        let kids = find_children(scene,&root.name);
        for kid in kids {
            if let Some(kid) = scene.get_view(&kid) {
                (kid.draw)(root,&ctx,&theme);
            }
        }
        scene.dirty = false;
    }
}
fn draw_generic_view(view: &View, ctx: &dyn DrawingContext, theme:&Theme) {
    ctx.fillRect(&view.bounds,theme.bg)
}
fn draw_root_view(view: &View, ctx: &dyn DrawingContext, theme:&Theme) {
    ctx.fillRect(&view.bounds,theme.panel_bg)
}
fn draw_button_view(view: &View, ctx: &dyn DrawingContext, theme: &Theme) {
    ctx.fillRect(&view.bounds, theme.bg);
    ctx.strokeRect(&view.bounds, theme.fg);
    ctx.fillText(&view.bounds, &view.title, theme.fg);
}
fn draw_panel_view(view: &View, ctx: &dyn DrawingContext, theme: &Theme) {
    ctx.fillRect(&view.bounds, theme.panel_bg);
}

#[cfg(test)]
mod tests {
    use std::sync::Once;
    use log::LevelFilter;
    use super::*;
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
    fn make_simple_view(name: &str) -> View {
        View {
            name:name.to_string(),
            title:name.to_string(),
            bounds: Bounds { x: 0, y: 0, w: 10, h: 10},
            visible:true,
            draw: draw_generic_view
        }
    }
    fn make_panel(name: &str, bounds: Bounds) -> View {
        View {
            name:name.to_string(),
            title: name.to_string(),
            bounds,
            visible:true,
            draw: draw_panel_view,
        }
    }
    fn make_button(name: &str, bounds: Bounds) -> View {
        View {
            name:name.to_string(),
            title: name.to_string(),
            bounds,
            visible:true,
            draw: draw_button_view
        }
    }
    #[test]
    fn test_geometry() {
        let bounds = Bounds { x: 0, y:0, w: 100, h:100};
        assert_eq!(bounds.contains(&Point::new(10,10)), true);
        assert_eq!(bounds.contains(&Point::new(-1,-1)),false);
    }
    #[test]
    fn basic_add_remove() {
        let mut scene: Scene = Scene::new();
        assert_eq!(scene.viewcount(), 1);
        let view: View = make_simple_view("foo");
        assert_eq!(scene.viewcount(), 1);
        scene.add_view(view);
        assert_eq!(scene.viewcount(), 2);
        assert_eq!(scene.has_view("foo"), true);
        let res: Option<View> = scene.remove_view("foo");
        assert_eq!(res.is_some(), true);
        assert_eq!(scene.viewcount(), 1);
        let res2: Option<View> = scene.remove_view( "bar");
        assert_eq!(res2.is_some(), false);
    }
    #[test]
    fn parent_child() {
        let mut scene: Scene = Scene::new();
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
        let mut scene: Scene = Scene::new();
        // add panel
        scene.add_view(make_panel("parent", Bounds { x: 10, y: 10, w: 100, h: 100}));
        // add button
        scene.add_view(make_button("child", Bounds { x: 10, y:10, w: 20, h: 20}));
        // connect
        connect_parent_child(&mut scene, "root","parent");
        connect_parent_child(&mut scene, "parent", "child");
        assert_eq!(pick_at(&mut scene, &Point { x: 5, y: 5 }).len(),1);
        assert_eq!(pick_at(&mut scene, &Point { x: 15, y: 15 }).len(),2);
        assert_eq!(pick_at(&mut scene, &Point { x: 25, y: 25 }).len(),3);
    }
    #[test]
    fn test_layout() {
        let mut scene: Scene = Scene::new();
        // add panel
        scene.add_view(make_panel("parent", Bounds { x: 10, y: 10, w: 100, h: 100}));
        // add button 1
        scene.add_view(make_button("button1", Bounds { x: 20, y: 20, w: 20, h: 20}));
        // add button 2
        scene.add_view(make_button("button2", Bounds { x: 20, y: 20, w: 20, h: 20}));
        // connect
        connect_parent_child(&mut scene, "parent", "button1");
        connect_parent_child(&mut scene, "parent", "button2");
        // layout
        layout_vbox(&mut scene, "parent");
        assert_eq!(get_bounds(&scene, "parent"), Some(Bounds { x: 10, y: 10, w: 100, h: 100}));
        assert_eq!(get_bounds(&scene, "button1"), Some(Bounds { x: 0, y: 0, w: 100, h: 20}));
        assert_eq!(get_bounds(&scene, "button2"), Some(Bounds { x: 0, y: 20, w: 100, h: 20}));
    }
    #[test]
    fn test_repaint() {
        let mut scene: Scene = Scene::new();
        // add panel
        scene.add_view(make_panel("parent", Bounds { x: 10, y: 10, w: 100, h: 100}));
        // add button 1
        scene.add_view(make_button("button1", Bounds { x: 20, y: 20, w: 20, h: 20}));
        // add button 2
        scene.add_view(make_button("button2", Bounds { x: 20, y: 20, w: 20, h: 20}));

        assert_eq!(scene.dirty,true);
        repaint(&mut scene);
        assert_eq!(scene.dirty,false);

    }
    #[test]
    fn test_events() {
        let mut scene = Scene::new();
        let mut handlers:Vec<Callback> = vec![];
        handlers.push(|event| {
            info!("got an event {:?}",event);
            if let Some(view) = event.scene.get_view_mut(event.target) {
                view.visible = false;
            }
            event.scene.dirty = true;
        });
        handlers.push(|event| {
            info!("got another event {:?}",event);
            if let Some(view) = event.scene.get_view_mut(event.target) {
                view.visible = false;
            }
            event.scene.dirty = true;
        });
        assert_eq!(scene.get_view("root").unwrap().visible,true);
        click_at(&mut scene, &handlers, Point::new(5,5));
        assert_eq!(scene.get_view("root").unwrap().visible,false);
    }
}

struct FakeDrawingContext {
    clip:Bounds,
}
impl DrawingContext for FakeDrawingContext {
    fn fillRect(&self, bounds: &Bounds, color: &str) {
    }

    fn strokeRect(&self, bounds: &Bounds, color: &str) {
    }

    fn fillText(&self, bounds: &Bounds, text: &str, color: &str) {
    }
}
