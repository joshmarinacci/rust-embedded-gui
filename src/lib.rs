#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate core;

use crate::geom::Size;
use crate::scene::Scene;
use crate::view::ViewId;
use alloc::string::String;
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::pixelcolor::Rgb565;
use geom::{Bounds, Point};
use gfx::DrawingContext;
use view::View;

pub mod button;
pub mod device;
pub mod geom;
pub mod gfx;
pub mod grid;
pub mod label;
pub mod layouts;
pub mod list_view;
pub mod panel;
pub mod scene;
pub mod test;
pub mod text_input;
pub mod toggle_button;
pub mod toggle_group;
pub mod util;
pub mod view;

pub struct DrawEvent<'a> {
    pub ctx: &'a mut dyn DrawingContext,
    pub theme: &'a Theme,
    pub focused: &'a Option<ViewId>,
    pub view: &'a mut View,
    pub bounds: &'a Bounds,
}

#[derive(Debug, Clone)]
pub enum Action {
    Generic,
    Command(String),
}
pub type DrawFn = fn(event: &mut DrawEvent);
pub type LayoutFn = fn(layout: &mut LayoutEvent);
pub type InputFn = fn(event: &mut GuiEvent) -> Option<Action>;

#[derive(Debug)]
pub struct Theme {
    pub bg: Rgb565,
    pub fg: Rgb565,
    pub panel_bg: Rgb565,
    pub selected_bg: Rgb565,
    pub selected_fg: Rgb565,
    pub font: MonoFont<'static>,
    pub bold_font: MonoFont<'static>,
}

pub type Callback = fn(event: &mut GuiEvent);

#[derive(Debug, Clone)]
pub enum KeyboardAction {
    Left,
    Right,
    Up,
    Down,
    Backspace,
    Return,
}
#[derive(Debug, Clone)]
pub enum EventType {
    Generic,
    Unknown,
    Tap(Point),
    Scroll(i32, i32),
    Keyboard(u8),
    KeyboardAction(KeyboardAction),
    Action(),
}
#[derive(Debug)]
pub struct GuiEvent<'a> {
    pub scene: &'a mut Scene,
    pub target: &'a ViewId,
    pub event_type: EventType,
    pub action: Option<Action>,
}

#[derive(Debug)]
pub struct LayoutEvent<'a> {
    pub scene: &'a mut Scene,
    pub target: &'a ViewId,
    pub space: Size,
    pub theme: &'a Theme,
}

impl<'a> LayoutEvent<'a> {
    pub(crate) fn layout_all_children(&mut self, name: &ViewId, space: Size) {
        let fixed_kids = self.scene.get_children_ids(name);
        for kid in &fixed_kids {
            self.layout_child(kid, space);
        }
    }
    pub(crate) fn layout_child(&mut self, kid: &ViewId, available_space: Size) {
        if let Some(view) = self.scene.get_view_mut(kid) {
            if let Some(layout) = view.layout {
                let mut pass = LayoutEvent {
                    target: kid,
                    space: available_space,
                    scene: self.scene,
                    theme: self.theme,
                };
                layout(&mut pass);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::button::make_button;
    use crate::gfx::TextStyle;
    use crate::scene::{click_at, draw_scene, event_at_focused, pick_at};
    use crate::test::MockDrawingContext;
    use crate::view::Align;
    use alloc::boxed::Box;
    use alloc::vec;
    use alloc::vec::Vec;
    use log::{LevelFilter, info};
    use std::sync::Once;
    use test_log::test;

    extern crate std;

    fn make_simple_view(name: &ViewId) -> View {
        View {
            name: name.clone(),
            title: name.to_string(),
            bounds: Bounds::new(0, 0, 10, 10),
            visible: true,
            draw: Some(|e| e.ctx.fill_rect(&e.view.bounds, &e.theme.bg)),
            input: None,
            state: None,
            layout: None,
            ..Default::default()
        }
    }
    fn layout_vbox(evt: &mut LayoutEvent) {
        if let Some(parent) = evt.scene.get_view_mut(evt.target) {
            let mut y = 0;
            let bounds = parent.bounds;
            let kids = evt.scene.get_children_ids(evt.target);
            for kid in kids {
                if let Some(ch) = evt.scene.get_view_mut(&kid) {
                    ch.bounds.position.x = 0;
                    ch.bounds.position.y = y;
                    ch.bounds.size.w = bounds.w();
                    y += ch.bounds.h();
                }
            }
        }
    }
    fn make_vbox(name: &ViewId, bounds: Bounds) -> View {
        View {
            name: name.clone(),
            title: name.to_string(),
            bounds,
            visible: true,
            draw: Some(|e| {
                e.ctx.fill_rect(&e.view.bounds, &e.theme.panel_bg);
            }),
            input: None,
            state: None,
            layout: Some(layout_vbox),
            ..Default::default()
        }
    }
    struct TestButtonState {
        drawn: bool,
        got_input: bool,
    }
    fn make_test_button(name: &ViewId) -> View {
        View {
            name: name.clone(),
            title: name.to_string(),
            bounds: Bounds::new(0, 0, 20, 20),
            visible: true,
            draw: Some(|e| {
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
            ..Default::default()
        }
    }
    fn make_text_box(name: &ViewId, title: &str) -> View {
        View {
            name: name.clone(),
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
            ..Default::default()
        }
    }
    fn draw_label_view(e: &mut DrawEvent) {
        e.ctx.fill_text(
            &e.view.bounds,
            &e.view.title,
            &TextStyle::new(&e.theme.font, &e.theme.fg),
        );
    }
    fn make_label(name: &ViewId) -> View {
        View {
            name: name.clone(),
            title: name.to_string(),
            bounds: Bounds::new(0, 0, 30, 20),
            visible: true,
            draw: Some(draw_label_view),
            input: None,
            state: None,
            layout: None,
            ..Default::default()
        }
    }
    fn get_bounds(scene: &Scene, name: &ViewId) -> Option<Bounds> {
        if let Some(view) = scene.keys.get(name) {
            Some(view.bounds)
        } else {
            None
        }
    }
    fn was_button_clicked(scene: &mut Scene, name: &ViewId) -> bool {
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
    fn was_button_drawn(scene: &mut Scene, name: &ViewId) -> bool {
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

    fn repaint(scene: &mut Scene) {
        let theme = MockDrawingContext::make_mock_theme();
        let mut ctx = MockDrawingContext::new(scene);
        draw_scene(scene, &mut ctx, &theme);
        scene.dirty_rect = Bounds::new_empty();
    }

    #[test]
    fn test_geometry() {
        let bounds = Bounds::new(0, 0, 100, 100);
        assert_eq!(bounds.contains(&Point::new(10, 10)), true);
        assert_eq!(bounds.contains(&Point::new(-1, -1)), false);

        let b2 = Bounds::new(140, 180, 80, 30);
        let b3 = Bounds::new(140, 180, 80, 30);
        // INFO - union Bounds { x: 140, y: 180, w: 80, h: 30 } Bounds { x: 140, y: 180, w: 80, h: 30 }
        assert_eq!(b2.union(b3), b2.clone());
    }
    #[test]
    fn basic_add_remove() {
        let mut scene: Scene = Scene::new_with_bounds(Bounds::new(0, 0, 100, 30));
        assert_eq!(scene.viewcount(), 1);
        let view = make_simple_view(&"foo".into());
        assert_eq!(scene.viewcount(), 1);
        scene.add_view(view);
        assert_eq!(scene.viewcount(), 2);
        assert!(scene.get_view(&"foo".into()).is_some());
        let res = scene.remove_view(&"foo".into());
        assert_eq!(res.is_some(), true);
        assert_eq!(scene.viewcount(), 1);
        let res2 = scene.remove_view(&"bar".into());
        assert_eq!(res2.is_some(), false);
    }
    #[test]
    fn parent_child() {
        let mut scene: Scene = Scene::new();
        let parent_id: ViewId = "parent".into();
        let parent = &parent_id;
        let child_id: ViewId = "child".into();
        let child = &child_id;
        scene.add_view(make_simple_view(parent));
        scene.add_view(make_simple_view(child));
        assert_eq!(scene.get_children_ids(parent).len(), 0);
        assert_eq!(scene.viewcount(), 3);
        scene.add_child(parent, child);
        assert_eq!(scene.get_children_ids(parent).len(), 1);
        scene.remove_child(parent, child);
        assert_eq!(scene.get_children_ids(parent).len(), 0);

        scene.add_child(parent, child);
        assert_eq!(scene.get_children_ids(parent).len(), 1);
        let child2 = make_simple_view(&"child2".into());
        scene.add_view_to_parent(child2, parent);
        assert_eq!(scene.get_children_ids(parent).len(), 2);
        assert_eq!(scene.viewcount(), 4);

        scene.remove_parent_and_children(parent);
        assert_eq!(scene.get_children_ids(parent).len(), 0);
        assert_eq!(scene.viewcount(), 1);
    }
    #[test]
    fn test_pick_at() {
        let mut scene: Scene = Scene::new();
        let vbox = make_vbox(&"parent".into(), Bounds::new(10, 10, 100, 100));

        let mut button = make_test_button(&ViewId::new("child"));
        button.bounds = Bounds::new(10, 10, 10, 10);

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
        let parent: ViewId = "parent".into();
        let theme = MockDrawingContext::make_mock_theme();
        let mut scene: Scene = Scene::new();
        // add panel
        scene.add_view(make_vbox(&parent, Bounds::new(10, 10, 100, 100)));
        // add button 1
        scene.add_view_to_parent(make_test_button(&ViewId::new("button1")), &parent);
        // add button 2
        scene.add_view_to_parent(make_label(&"button2".into()), &parent);
        // layout
        let space = scene.bounds.size.clone();
        layout_vbox(&mut LayoutEvent {
            scene: &mut scene,
            target: &"parent".into(),
            theme: &theme,
            space: space,
        });
        assert_eq!(
            get_bounds(&scene, &"parent".into()),
            Some(Bounds::new(10, 10, 100, 100))
        );
        assert_eq!(
            get_bounds(&scene, &"button1".into()),
            Some(Bounds::new(0, 0, 100, 20)),
        );
        assert_eq!(
            get_bounds(&scene, &"button2".into()),
            Some(Bounds::new(0, 20, 100, 20))
        );
    }
    #[test]
    fn test_repaint() {
        let mut scene = Scene::new();
        // add panel
        scene.add_view(make_vbox(&"parent".into(), Bounds::new(10, 10, 100, 100)));
        // add button 1
        scene.add_view(make_test_button(&ViewId::new("button1")));
        // add button 2
        scene.add_view(make_test_button(&ViewId::new("button2")));

        assert_eq!(scene.dirty, true);
        repaint(&mut scene);
        assert_eq!(scene.dirty, false);
    }
    #[test]
    fn test_events() {
        let mut scene: Scene = Scene::new();
        let mut handlers: Vec<Callback> = vec![];
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
        assert_eq!(scene.get_view(&"root".into()).unwrap().visible, true);
        click_at(&mut scene, &handlers, Point::new(5, 5));
        assert_eq!(scene.get_view(&"root".into()).unwrap().visible, false);
    }
    fn handle_toggle_button_input(event: &mut GuiEvent) -> Option<Action> {
        // info!("view clicked {:?}", event.event_type);
        if let Some(view) = event.scene.get_view_mut(event.target) {
            view.state.insert(Box::new(String::from("enabled")));
        }
        None
    }
    #[test]
    fn test_toggle_button() {
        let mut scene = Scene::new();
        // add toggle button
        let button = View {
            name: ViewId::new("toggle"),
            title: String::from("Off"),
            visible: true,
            bounds: Bounds::new(10, 10, 20, 20),
            draw: Some(|e| {
                if let Some(state) = &e.view.state {
                    if let Some(state) = state.downcast_ref::<String>() {
                        if state == "enabled" {
                            e.ctx.fill_rect(&e.view.bounds, &e.theme.fg);
                            e.ctx.stroke_rect(&e.view.bounds, &e.theme.bg);
                            let style = TextStyle::new(&e.theme.font, &e.theme.bg)
                                .with_halign(Align::Center);
                            e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
                        } else {
                            e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
                            e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
                            let style = TextStyle::new(&e.theme.font, &e.theme.fg)
                                .with_halign(Align::Center);
                            e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
                        }
                    }
                }
            }),
            input: Some(handle_toggle_button_input),
            state: Some(Box::new(String::from("disabled"))),
            layout: None,
            ..Default::default()
        };
        scene.add_view_to_root(button);
        // repaint
        repaint(&mut scene);
        assert_eq!(scene.get_view(&"toggle".into()).unwrap().visible, true);
        assert_eq!(
            &scene
                .get_view(&"toggle".into())
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
        let handlers = vec![];
        click_at(&mut scene, &handlers, Point::new(15, 15));
        // confirm toggle button state has changed to enabled
        assert_eq!(
            &scene
                .get_view(&"toggle".into())
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
        let mut scene = Scene::new();

        // create button 1
        let mut button1 = make_test_button(&ViewId::new("button1"));
        button1.visible = true;
        scene.add_view_to_root(button1);

        // create button 2
        let mut button2 = make_test_button(&ViewId::new("button2"));
        button2.bounds.position.x = 100;
        // make button 2 invisible
        button2.visible = false;
        scene.add_view_to_root(button2);

        assert_eq!(was_button_clicked(&mut scene, &"button1".into()), false);
        assert_eq!(was_button_drawn(&mut scene, &"button1".into()), false);
        assert_eq!(was_button_drawn(&mut scene, &"button2".into()), false);

        // repaint. only button 1 should get drawn
        repaint(&mut scene);
        assert_eq!(scene.dirty, false);
        assert_eq!(was_button_drawn(&mut scene, &"button1".into()), true);
        assert_eq!(was_button_drawn(&mut scene, &"button2".into()), false);

        let mut handlers: Vec<Callback> = vec![];
        handlers.push(|e| {
            info!("clicked on {}", e.target);
            if let Some(view) = e.scene.get_view_mut(&"button2".into()) {
                view.visible = true;
                e.scene.dirty = true;
            }
        });

        // tap button 1
        assert_eq!(scene.dirty, false);
        click_at(&mut scene, &handlers, Point::new(15, 15));
        assert_eq!(was_button_clicked(&mut scene, &"button1".into()), true);
        // confirm dirty
        assert_eq!(scene.dirty, true);

        // this time both buttons should be drawn
        repaint(&mut scene);
        assert_eq!(scene.dirty, false);
        assert_eq!(was_button_drawn(&mut scene, &"button1".into()), true);
        assert_eq!(was_button_drawn(&mut scene, &"button2".into()), true);
    }
    #[test]
    fn test_keyboard_events() {
        // make scene
        let mut scene: Scene = Scene::new();

        // make text box
        let text_box = make_text_box(&ViewId::new("textbox1"), "foo");
        scene.add_view_to_root(text_box);
        // confirm text is correct
        assert_eq!(get_view_title(&scene, ViewId::new("textbox1")), "foo");
        // set text box as focused
        scene.focused = Some("textbox1".into());

        // send keyboard event
        event_at_focused(&mut scene, &EventType::Keyboard(b'X'));
        // confirm text is updated
        assert_eq!(get_view_title(&scene, ViewId::new("textbox1")), "fooX");
    }

    #[test]
    fn test_draw2() {
        let mut scene = Scene::new();
        let view = View {
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
            ..Default::default()
        };

        scene.add_view_to_root(view);
        repaint(&mut scene);
    }

    #[test]
    fn test_cliprect() {
        // make scene
        let mut scene = Scene::new();
        // add button
        let button = make_button(&"button".into(), "Button").position_at(20, 20);
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
        click_at(&mut scene, &vec![], Point::new(30, 30));
        // check that dirty area is just for the button
        assert_eq!(scene.dirty, true);
        assert_eq!(
            scene.dirty_rect,
            scene.get_view(&"button".into()).unwrap().bounds
        );
        // draw
        repaint(&mut scene);
        assert_eq!(scene.dirty, false);
        assert_eq!(scene.dirty_rect.is_empty(), true);
        // check that button was redrawn
    }

    fn get_view_title(scene: &Scene, name: ViewId) -> String {
        scene.get_view(&name).unwrap().title.clone()
    }
}
