pub struct Scene {
    views:Vec<View>
}

impl Scene {
    pub(crate) fn connectioncount(&self) -> i32 {
        0
    }
    pub(crate) fn has_view(&self, name: &str) -> bool {
        let view = self.views.iter().find(|v|v.name.eq(name));
        return view.is_some();
    }
    pub(crate) fn viewcount(&self) -> usize {
        self.views.len()
    }
}

impl Scene {
    pub(crate) fn new() -> Scene {
        let root = View {
            name:"root".to_string()
        };
        Scene {
            views:vec![root]
        }
    }
}

pub struct View {
    name:String,
}

struct Point {
    x: i32,
    y: i32,
}

fn remove_view(scene: &mut Scene, name: &str) -> Option<View> {
    if let Some(n) = scene.views.iter().position(|v|v.name.eq(name)) {
        return Some(scene.views.remove(n));
    }
    None
}

fn add_view(scene: &mut Scene, view: View) {
    scene.views.push(view)
}

fn make_simple_view(name: &str) -> View {
    View {
        name:name.to_string()
    }
}

fn connect_parent_child(scene: &mut Scene, parent: &str, child: &str) {
    todo!()
}
fn remove_parent_child(scene: &mut Scene, parent: &str, child: &str) {
    todo!()
}

fn pick_at(scene: &mut Scene, pt: Point) -> Vec<&View> {
    todo!()
}
#[derive(Debug, PartialEq)]
struct Bounds {
    x: i32,
    y: i32,
    w: i32,
    h: i32
}

fn get_bounds(scene: &Scene, name: &str) -> Bounds {
    todo!()
}

fn layout_vbox(scene: &mut Scene, name: &str) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(get_children(&mut scene, "parent").len(), 0);
        assert_eq!(scene.viewcount(), 3);

        connect_parent_child(&mut scene, "parent", "child");
        assert_eq!(scene.connectioncount(), 1);
        assert_eq!(get_children(&mut scene, "parent").len(), 1);

        remove_parent_child(&mut scene, "parent", "child");
        assert_eq!(scene.connectioncount(), 0);
        assert_eq!(get_children(&mut scene, "parent").len(), 0);
    }

    fn get_children<'a>(scene: &'a mut Scene, name: &str) -> Vec<&'a View> {
        todo!()
    }

    #[test]
    fn test_pick_at() {
        let mut scene: Scene = Scene::new();
        // add panel
        add_view(&mut scene, make_panel("parent", Bounds { x: 10, y: 10, w: 100, h: 100}));
        // add button
        add_view(&mut scene, make_button("child", Bounds { x: 20, y:20, w: 20, h: 20}));
        // connect
        connect_parent_child(&mut scene, "parent", "child");
        assert_eq!(pick_at(&mut scene, Point { x: 5, y: 5 }).len(),0);
        assert_eq!(pick_at(&mut scene, Point { x: 15, y: 15 }).len(),1);
        assert_eq!(pick_at(&mut scene, Point { x: 25, y: 25 }).len(),2);
    }

    fn make_panel(name: &str, bounds: Bounds) -> View {
        View {
            name:name.to_string()
        }
    }
    fn make_button(name: &str, bounds: Bounds) -> View {
        View {
            name:name.to_string()
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
        assert_eq!(get_bounds(&scene, "parent"), Bounds { x: 10, y: 10, w: 100, h: 100});
        assert_eq!(get_bounds(&scene, "button1"), Bounds { x: 0, y: 0, w: 100, h: 20});
        assert_eq!(get_bounds(&scene, "button2"), Bounds { x: 0, y: 20, w: 100, h: 20});
        let views: Vec<&View> = pick_at(&mut scene, Point { x: 5, y: 5 });
    }
}
