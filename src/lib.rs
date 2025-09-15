#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate core;

use crate::scene::Scene;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;
use geom::{Bounds, Point};
use log::info;
use view::View;

pub mod comps;
pub mod form;
pub mod geom;
pub mod scene;
pub mod toggle_button;
pub mod toggle_group;
pub mod view;

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
    pub fn new(font: &'a F, color: &'a C) -> TextStyle<'a, C, F> {
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
            valign: self.valign,
        }
    }
    pub fn with_halign(&self, halign: HAlign) -> Self {
        TextStyle {
            color: self.color,
            font: self.font,
            underline: self.underline,
            halign,
            valign: self.valign,
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
    pub view: &'a mut View<C, F>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comps::make_button;
    use crate::scene::{click_at, draw_scene, event_at_focused, pick_at};
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
            draw: Some(|e|{
                e.ctx.fill_rect(&e.view.bounds, &e.theme.bg)
            }),
            input: None,
            state: None,
            layout: None,
        }
    }
    fn layout_vbox<C, F>(evt: &mut LayoutEvent<C, F>) {
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
    fn make_vbox<C, F>(name: &str, bounds: Bounds) -> View<C, F> {
        View {
            name: name.to_string(),
            title: name.to_string(),
            bounds,
            visible: true,
            draw: Some(|e| {
                e.ctx.fill_rect(&e.view.bounds, &e.theme.panel_bg);
            }),
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
            draw:Some(|e| {
                if let Some(state) = &mut e.view.state {
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
        e: &mut DrawEvent<C, F>,
    ) {
        e.ctx.fill_text(
            &e.view.bounds,
            &e.view.title,
            &TextStyle::new(&e.theme.font, &e.theme.fg),
        );
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
        let button: View<String, String> = View {
            name: String::from("toggle"),
            title: String::from("Off"),
            visible: true,
            bounds: Bounds {
                x: 10,
                y: 10,
                w: 20,
                h: 20,
            },
            draw: Some(|e| {
                if let Some(state) = &e.view.state {
                    if let Some(state) = state.downcast_ref::<String>() {
                        if state == "enabled" {
                            e.ctx.fill_rect(&e.view.bounds, &e.theme.fg);
                            e.ctx.stroke_rect(&e.view.bounds, &e.theme.bg);
                            let style =
                                TextStyle::new(&e.theme.font, &e.theme.bg).with_halign(HAlign::Center);
                            e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
                        } else {
                            e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
                            e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
                            let style =
                                TextStyle::new(&e.theme.font, &e.theme.fg).with_halign(HAlign::Center);
                            e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
                        }
                    }
                }
            }),
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
    fn test_keyboard_events() {
        // make scene
        initialize();
        let mut scene:Scene<String,String> = Scene::new();

        // make text box
        let text_box = make_text_box("textbox1", "foo");
        scene.add_view_to_root(text_box);
        // confirm text is correct
        assert_eq!(get_view_title(&scene, "textbox1"), "foo");
        // set text box as focused
        scene.focused = Some("textbox1".into());

        // send keyboard event
        event_at_focused(&mut scene, EventType::Keyboard(b'X'));
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
            draw: Some(|e| {
                let mut color = &e.theme.fg;
                if e.focused.is_some() && e.view.name.eq(e.focused.as_ref().unwrap()) {
                    color = &e.theme.bg;
                }
                e.ctx.fill_rect(&e.view.bounds, color);
            }),
            state: None,
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

    fn fill_text(&mut self, _bounds: &Bounds, _text: &str, _style: &TextStyle<String, String>) {}
}
