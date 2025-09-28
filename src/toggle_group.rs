use crate::geom::{Bounds, Point};
use crate::gfx::{draw_centered_text, DrawingContext};
use crate::view::Flex::{Intrinsic, Resize};
use crate::view::{View, ViewId};
use crate::{Action, DrawEvent, EventType, GuiEvent, LayoutEvent};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use core::option::Option::Some;
use hashbrown::Equivalent;

pub fn make_toggle_group(name: &ViewId, data: Vec<&str>, selected: usize) -> View {
    View {
        name: name.clone(),
        title: name.as_str().into(),
        bounds: Bounds::new(0, 0, (data.len() * 60) as i32, 30),
        state: Some(SelectOneOfState::new_with(data, selected)),
        input: Some(input_toggle_group),
        layout: Some(layout_toggle_group),
        draw: Some(draw_toggle_group),
        visible: true,
        h_flex: Resize,
        v_flex: Intrinsic,
        .. Default::default()
    }
}

pub struct SelectOneOfState {
    pub items: Vec<String>,
    pub selected: usize,
}

impl SelectOneOfState {
    pub fn new_with(items: Vec<&str>, selected: usize) -> Box<dyn Any> {
        Box::new(SelectOneOfState {
            items: items.iter().map(|s| s.to_string()).collect(),
            selected,
        })
    }
}

fn input_toggle_group(e: &mut GuiEvent) -> Option<Action> {
    match &e.event_type {
        EventType::Tap(pt) => {
            e.scene.mark_dirty_view(e.target);
            e.scene.set_focused(e.target);
            if let Some(view) = e.scene.get_view_mut(e.target) {
                let bounds = view.bounds;
                if let Some(state) = view.get_state::<SelectOneOfState>() {
                    let cell_width = bounds.size.w/ (state.items.len() as i32);
                    let x = pt.x - bounds.x();
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

fn draw_toggle_group(e: &mut DrawEvent) {
    let bounds = e.view.bounds;
    e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
    let name = e.view.name.clone();
    if let Some(state) = e.view.get_state::<SelectOneOfState>() {
        let cell_width = bounds.size.w/ (state.items.len() as i32);
        for (i, item) in state.items.iter().enumerate() {
            let (bg, fg) = if i == state.selected {
                (&e.theme.selected_bg, &e.theme.selected_fg)
            } else {
                (&e.theme.bg, &e.theme.fg)
            };
            let bds = Bounds::new(
                bounds.position.x+ (i as i32) * cell_width + 1,
                bounds.y(),
                cell_width - 1,
                bounds.h(),
            );
            // draw background only if selected
            if i == state.selected {
                e.ctx.fill_rect(&bds, bg);
                if let Some(focused) = e.focused {
                    if focused == &name {
                        e.ctx.stroke_rect(&bds.contract(2), fg);
                    }
                }
            }

            // draw text
            draw_centered_text(e.ctx, item, &bds, &e.theme.font, fg);

            // draw left edge except for the first one
            if i != 0 {
                let x = bounds.position.x+ (i as i32) * cell_width;
                e.ctx.line(
                    &Point::new(x, bounds.y()),
                    &Point::new(x, bounds.position.y+ bounds.size.h- 1),
                    &e.theme.fg,
                );
            }
        }
    }
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
}

pub fn layout_toggle_group(pass: &mut LayoutEvent) {
    if let Some(view) = pass.scene.get_view_mut(&pass.target) {
        let char_size = pass.theme.font.character_size;
        let mut height = char_size.height + (char_size.height / 2) * 2; // padding
        if view.h_flex == Resize {
            view.bounds.size.w = pass.space.w;
        }
        if view.h_flex == Intrinsic {
            view.bounds.size.w = 50;
        }
        if view.v_flex == Resize {
            view.bounds.size.h = pass.space.h;
        }
        if view.v_flex == Intrinsic {
            view.bounds.size.h = height as i32;
        }
    }
    pass.layout_all_children(&pass.target.clone(),pass.space);
}
mod tests {
    use crate::geom::{Bounds, Point};
    use crate::scene::{click_at, draw_scene, layout_scene, Scene};
    use crate::test::MockDrawingContext;
    use crate::toggle_group::{make_toggle_group, SelectOneOfState};
    use crate::view::ViewId;
    use alloc::vec;

    #[test]
    fn test_toggle_group() {
        let theme = MockDrawingContext::make_mock_theme();
        let mut scene: Scene = Scene::new_with_bounds(Bounds::new(0, 0, 100, 240));
        {
            let group = make_toggle_group(&ViewId::new("group"), vec!["A", "BB", "CCC"], 0);
            scene.add_view_to_root(group);
        }
        layout_scene(&mut scene, &theme);

        {
            let mut group = scene.get_view_mut(&ViewId::new("group")).unwrap();
            assert_eq!(group.name.as_str(), "group");
            assert_eq!(group.bounds, Bounds::new(0, 0, 100, 13 + 7));
            let state = &mut group.get_state::<SelectOneOfState>().unwrap();
            assert_eq!(state.items, vec!["A", "BB", "CCC"]);
            assert_eq!(state.selected, 0);
        }

        click_at(&mut scene, &vec![], Point::new(50, 10));

        {
            let state = &mut scene.get_view_state::<SelectOneOfState>(&ViewId::new("group")).unwrap();
            assert_eq!(state.items, vec!["A", "BB", "CCC"]);
            assert_eq!(state.selected, 1);
        }

        let mut ctx = MockDrawingContext::new(&scene);
        assert_eq!(scene.dirty, true);
        draw_scene(&mut scene, &mut ctx, &theme);
        assert_eq!(scene.dirty, false);
    }
}

