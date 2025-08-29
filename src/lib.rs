#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate core;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use hashbrown::HashMap;
use log::info;


pub struct Scene {
    keys: HashMap<String, View>,
    connections: Vec<Connection>
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
    pub fn get_children(&mut self, name: &str) -> Vec<&mut View> {
        todo!()
    }
}

impl Scene {
    pub(crate) fn new() -> Scene {
        let root = View {
            name:"root".to_string(),
            bounds: Bounds { x:0, y:0, w: 200, h: 200},
        };
        let mut keys:HashMap<String, View> = HashMap::new();
        keys.insert(String::from("root"),root);
        Scene {
            keys: keys,
            connections: vec![]
        }
    }
}

pub struct View {
    name:String,
    bounds: Bounds
}

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub(crate) fn new(x: i32, y: i32) -> Point {
        Point { x, y, }
    }
}

fn remove_view(scene: &mut Scene, name: &str) -> Option<View> {
    scene.keys.remove(name)
}

fn add_view(scene: &mut Scene, view: View) {
    scene.keys.insert(view.name.clone(),view);
}

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

fn pick_at(scene: &mut Scene, pt: &Point) -> Vec<String> {
    pick_at_view(scene, pt, "root")
}
fn pick_at_view(scene: &Scene, pt: &Point, name:&str) -> Vec<String> {
    let mut coll:Vec<String> = vec![];
    if let Some(root) = scene.keys.get(name) {
        // info!("picking view {} {:?} {:?} {}",name, root.bounds, pt, root.bounds.contains(pt));
        if root.bounds.contains(pt) {
            coll.push(root.name.clone());
            let pt2 = Point {
                x:pt.x-root.bounds.x,
                y:pt.y-root.bounds.y,
            };
            // info!("pt2 = {:?}",pt2);
            for con in &scene.connections {
                if con.parent == root.name {
                    let mut coll2 = pick_at_view(scene, &pt2, &con.child);
                    coll.append(&mut coll2);
                }
            }
        }
    }
    coll
}
#[derive(Debug, PartialEq, Copy, Clone)]
struct Bounds {
    x: i32,
    y: i32,
    w: i32,
    h: i32
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

fn get_bounds(scene: &Scene, name: &str) -> Option<Bounds> {
    if let Some(view) = scene.keys.get(name) {
        return Some(view.bounds);
    }
    None
}

fn layout_vbox(scene: &mut Scene, name: &str) {
    let parent = scene.keys.get_mut(name);
    if let Some(parent) = parent {
        let mut y = 0;
        let bounds = parent.bounds.clone();
        for con in &scene.connections {
            if con.parent == name {
                if let Some(ch) = scene.keys.get_mut(&con.child.clone()) {
                    ch.bounds.x = 0;
                    ch.bounds.y = y;
                    ch.bounds.w = bounds.w;
                    y += ch.bounds.h;
                }
            }
        }
    }
}
fn get_child_count(scene: &mut Scene, name: &str) -> usize {
    let conn:Vec<&Connection> = scene.connections.iter().filter(|c|c.parent == name).collect();
    conn.len()
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
        add_view(&mut scene, view);
        assert_eq!(scene.viewcount(), 2);
        assert_eq!(scene.has_view("foo"), true);
        let res: Option<View> = remove_view(&mut scene, "foo");
        assert_eq!(res.is_some(), true);
        assert_eq!(scene.viewcount(), 1);
        let res2: Option<View> = remove_view(&mut scene, "bar");
        assert_eq!(res2.is_some(), false);
    }

    #[test]
    fn parent_child() {
        let mut scene: Scene = Scene::new();
        add_view(&mut scene, make_simple_view("parent"));
        add_view(&mut scene, make_simple_view("child"));
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
        add_view(&mut scene, make_panel("parent", Bounds { x: 10, y: 10, w: 100, h: 100}));
        // add button
        add_view(&mut scene, make_button("child", Bounds { x: 10, y:10, w: 20, h: 20}));
        // connect
        connect_parent_child(&mut scene, "root","parent");
        connect_parent_child(&mut scene, "parent", "child");
        assert_eq!(pick_at(&mut scene, &Point { x: 5, y: 5 }).len(),1);
        assert_eq!(pick_at(&mut scene, &Point { x: 15, y: 15 }).len(),2);
        assert_eq!(pick_at(&mut scene, &Point { x: 25, y: 25 }).len(),3);
    }

    fn make_simple_view(name: &str) -> View {
        View {
            name:name.to_string(),
            bounds: Bounds { x: 0, y: 0, w: 10, h: 10}
        }
    }

    fn make_panel(name: &str, bounds: Bounds) -> View {
        View {
            name:name.to_string(),
            bounds
        }
    }
    fn make_button(name: &str, bounds: Bounds) -> View {
        View {
            name:name.to_string(),
            bounds,
        }
    }

    #[test]
    fn test_layout() {
        let mut scene: Scene = Scene::new();
        // add panel
        add_view(&mut scene, make_panel("parent", Bounds { x: 10, y: 10, w: 100, h: 100}));
        // add button 1
        add_view(&mut scene, make_button("button1", Bounds { x: 20, y: 20, w: 20, h: 20}));
        // add button 2
        add_view(&mut scene, make_button("button2", Bounds { x: 20, y: 20, w: 20, h: 20}));
        // connect
        connect_parent_child(&mut scene, "parent", "button1");
        connect_parent_child(&mut scene, "parent", "button2");
        // layout
        layout_vbox(&mut scene, "parent");
        assert_eq!(get_bounds(&scene, "parent"), Some(Bounds { x: 10, y: 10, w: 100, h: 100}));
        assert_eq!(get_bounds(&scene, "button1"), Some(Bounds { x: 0, y: 0, w: 100, h: 20}));
        assert_eq!(get_bounds(&scene, "button2"), Some(Bounds { x: 0, y: 20, w: 100, h: 20}));
        // let views: Vec<&View> = pick_at(&mut scene, Point { x: 5, y: 5 });
    }
}
