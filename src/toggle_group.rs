use crate::geom::Bounds;
use crate::{Action, DrawingContext, EventType, GuiEvent, HAlign, Theme, View};
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
        draw: Some(draw_toggle_group),
        input: Some(input_toggle_group),
        layout: None,
        draw2: None,
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
                    let n = x / cell_width as i32;
                    if n >= 0 && n < state.items.len() as i32 {
                        state.selected = n as usize;
                        return Some(Action::Command(
                            state.items[state.selected as usize].clone(),
                        ));
                    }
                }
            }
        }
        _ => {

        }
    }
    None
}

fn draw_toggle_group<C, F>(
    view: &mut View<C, F>,
    ctx: &mut dyn DrawingContext<C, F>,
    theme: &Theme<C, F>,
) {
    let bounds = view.bounds.clone();
    ctx.fill_rect(&view.bounds, &theme.bg);
    ctx.stroke_rect(&view.bounds, &theme.fg);
    if let Some(state) = view.get_state::<SelectOneOfState>() {
        let cell_width = bounds.w / (state.items.len() as i32);
        for (i, item) in state.items.iter().enumerate() {
            let (fill, color) = if i == state.selected {
                (&theme.fg, &theme.bg)
            } else {
                (&theme.bg, &theme.fg)
            };
            let bds = Bounds::new(
                bounds.x + (i as i32) * cell_width,
                bounds.y,
                cell_width,
                bounds.h,
            );
            ctx.fill_rect(&bds, fill);
            ctx.stroke_rect(&bds, &theme.fg);
            ctx.fill_text(&bds, item, color,&HAlign::Center);
        }
    }
}



mod tests {
    use crate::geom::{Bounds, Point};
    use crate::toggle_group::{make_toggle_group, SelectOneOfState};
    use crate::{click_at, draw_scene, layout_scene, MockDrawingContext, Scene, Theme};
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
            let group = make_toggle_group("group",vec!["A","B","C"],0);
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

        assert_eq!(scene.dirty,true);
        draw_scene(&mut scene, &mut ctx, &theme);
        assert_eq!(scene.dirty,false);

    }

}