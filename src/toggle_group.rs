use crate::geom::Bounds;
use crate::view::View;
use crate::{Action, DrawEvent, DrawingContext, EventType, GuiEvent, HAlign, TextStyle, Theme};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use core::option::Option::Some;

pub fn make_toggle_group<C, F>(name: &str, data: Vec<&str>, selected: usize) -> View<C, F> {
    View {
        name: name.into(),
        title: name.into(),
        bounds: Bounds::new(0, 0, (data.len() * 60) as i32, 30),
        state: Some(SelectOneOfState::new_with(data, selected)),
        input: Some(input_toggle_group),
        layout: None,
        draw: Some(draw_toggle_group),
        visible: true,
    }
}

pub struct SelectOneOfState {
    pub items: Vec<String>,
    pub selected: usize,
}

impl SelectOneOfState {
    fn new_with(items: Vec<&str>, selected: usize) -> Box<dyn Any> {
        Box::new(SelectOneOfState {
            items: items.iter().map(|s| s.to_string()).collect(),
            selected,
        })
    }
}

fn input_toggle_group<C, F>(event: &mut GuiEvent<C, F>) -> Option<Action> {
    match &event.event_type {
        EventType::Tap(pt) => {
            event.scene.mark_dirty_view(event.target);
            if let Some(view) = event.scene.get_view_mut(event.target) {
                let bounds = view.bounds;
                if let Some(state) = view.get_state::<SelectOneOfState>() {
                    let cell_width = bounds.w / (state.items.len() as i32);
                    let x = pt.x - bounds.x;
                    let n = x / cell_width;
                    if n >= 0 && n < state.items.len() as i32 {
                        state.selected = n as usize;
                        return Some(Action::Command(state.items[state.selected].clone()));
                    }
                }
            }
        }
        _ => {}
    }
    None
}

fn draw_toggle_group<C, F>(e: &mut DrawEvent<C, F>) {
    let bounds = e.view.bounds;
    e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
    if let Some(state) = e.view.get_state::<SelectOneOfState>() {
        let cell_width = bounds.w / (state.items.len() as i32);
        for (i, item) in state.items.iter().enumerate() {
            let (fill, color) = if i == state.selected {
                (&e.theme.fg, &e.theme.bg)
            } else {
                (&e.theme.bg, &e.theme.fg)
            };
            let bds = Bounds::new(
                bounds.x + (i as i32) * cell_width,
                bounds.y,
                cell_width,
                bounds.h,
            );
            e.ctx.fill_rect(&bds, fill);
            e.ctx.stroke_rect(&bds, &e.theme.fg);
            let style = TextStyle::new(&e.theme.font, color).with_halign(HAlign::Center);
            e.ctx.fill_text(&bds, item, &style);
        }
    }
}

mod tests {
    use crate::geom::{Bounds, Point};
    use crate::scene::{Scene, click_at, draw_scene, layout_scene};
    use crate::toggle_group::{SelectOneOfState, make_toggle_group};
    use crate::{MockDrawingContext, Theme};
    use alloc::string::String;
    use alloc::vec;

    #[test]
    fn test_toggle_group() {
        let theme: Theme<String, String> = Theme {
            bg: "white".into(),
            fg: "black".into(),
            panel_bg: "grey".into(),
            font: "plain".into(),
            bold_font: "bold".into(),
        };
        let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));
        {
            let group = make_toggle_group("group", vec!["A", "B", "C"], 0);
            scene.add_view_to_root(group);
        }
        layout_scene(&mut scene);

        {
            let mut group = scene.get_view_mut("group").unwrap();
            assert_eq!(group.name, "group");
            assert_eq!(group.bounds, Bounds::new(0, 0, 180, 30));
            let state = &mut group.get_state::<SelectOneOfState>().unwrap();
            assert_eq!(state.items, vec!["A", "B", "C"]);
            assert_eq!(state.selected, 0);
        }

        click_at(&mut scene, &vec![], Point::new(100, 10));

        {
            let state = &mut scene.get_view_state::<SelectOneOfState>("group").unwrap();
            assert_eq!(state.items, vec!["A", "B", "C"]);
            assert_eq!(state.selected, 1);
        }

        let mut ctx: MockDrawingContext<String, String> = MockDrawingContext {
            bg: String::new(),
            font: String::new(),
            clip: scene.dirty_rect,
        };

        assert_eq!(scene.dirty, true);
        draw_scene(&mut scene, &mut ctx, &theme);
        assert_eq!(scene.dirty, false);
    }
}
