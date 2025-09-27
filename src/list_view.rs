use crate::geom::{Bounds, Point};
use crate::gfx::{DrawingContext, HAlign, TextStyle, draw_centered_text};
use crate::view::{View, ViewId};
use crate::{Action, DrawEvent, EventType, GuiEvent, KeyboardAction, LayoutEvent};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use core::option::Option::Some;
use hashbrown::Equivalent;
use log::info;

pub fn make_list_view(name: &'static str, data: Vec<&str>, selected: usize) -> View {
    View {
        name: ViewId::new(name),
        title: name.into(),
        bounds: Bounds::new(0, 0, 100, (data.len() * 30) as i32),
        state: Some(SelectOneOf::new_with(data, selected)),
        input: Some(input_list),
        layout: Some(layout_list),
        draw: Some(draw_list),
        visible: true,
        .. Default::default()
    }
}

pub struct SelectOneOf {
    pub items: Vec<String>,
    pub selected: usize,
}

impl SelectOneOf {
    pub fn select_next(&mut self) {
        if self.selected < self.items.len() - 1 {
            self.selected += 1;
        }
    }
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
}

impl SelectOneOf {
    pub fn new_with(items: Vec<&str>, selected: usize) -> Box<dyn Any> {
        Box::new(SelectOneOf {
            items: items.iter().map(|s| s.to_string()).collect(),
            selected,
        })
    }
}

fn input_list(e: &mut GuiEvent) -> Option<Action> {
    match &e.event_type {
        EventType::Tap(pt) => {
            e.scene.mark_dirty_view(e.target);
            e.scene.set_focused(e.target);
            if let Some(view) = e.scene.get_view_mut(e.target) {
                let bounds = view.bounds;
                if let Some(state) = view.get_state::<SelectOneOf>() {
                    let cell_height = bounds.h() / (state.items.len() as i32);
                    let y = pt.y - bounds.y();
                    let n = y / cell_height;
                    if n >= 0 && n < state.items.len() as i32 {
                        state.selected = n as usize;
                        return Some(Action::Command(state.items[state.selected].clone()));
                    }
                }
            }
        }
        EventType::Scroll(x, y) => {
            e.scene.mark_dirty_view(e.target);
            if let Some(state) = e.scene.get_view_state::<SelectOneOf>(e.target) {
                if *y > 0 {
                    state.select_next();
                }
                if *y < 0 {
                    state.select_prev();
                }
            }
        }
        EventType::KeyboardAction(action) => {
            e.scene.mark_dirty_view(e.target);
            if let Some(state) = e.scene.get_view_state::<SelectOneOf>(e.target) {
                match action {
                    KeyboardAction::Up => state.select_prev(),
                    KeyboardAction::Down => state.select_next(),
                    KeyboardAction::Return => {
                        info!("firmly selecting the item");
                        return Some(Action::Command(
                            state.items[state.selected as usize].clone(),
                        ));
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    None
}

fn draw_list(e: &mut DrawEvent) {
    let bounds = e.view.bounds;
    e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
    let name = e.view.name.clone();
    if let Some(state) = e.view.get_state::<SelectOneOf>() {
        let cell_height = bounds.h() / (state.items.len() as i32);
        for (i, item) in state.items.iter().enumerate() {
            let (bg, fg) = if i == state.selected {
                (&e.theme.selected_bg, &e.theme.selected_fg)
            } else {
                (&e.theme.bg, &e.theme.fg)
            };
            let bds = Bounds::new(
                bounds.x(),
                bounds.y() + (i as i32) * cell_height + 1,
                bounds.w(),
                cell_height - 1,
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
        }
    }
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
}

fn layout_list(e: &mut LayoutEvent) {
    if let Some(state) = e.scene.get_view_state::<SelectOneOf>(e.target) {
        let ch = e.theme.font.character_size;
        let height = state.items.len() as u32 * ch.height * 2;
        if let Some(view) = e.scene.get_view_mut(e.target) {
            view.bounds = Bounds::new(view.bounds.x(), view.bounds.y(), view.bounds.w(), height as i32);
        }
    }
}
mod tests {
    use crate::geom::{Bounds, Point};
    use crate::scene::{Scene, click_at, draw_scene, layout_scene};
    use crate::test::MockDrawingContext;
    use crate::toggle_group::{SelectOneOfState, make_toggle_group};
    use alloc::vec;
    use crate::view::ViewId;

    #[test]
    fn test_list_view() {
        let theme = MockDrawingContext::make_mock_theme();
        let mut scene: Scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));
        {
            let list = make_toggle_group(&ViewId::new("listview"), vec!["A", "BB", "CCC"], 0);
            scene.add_view_to_root(list);
        }
        layout_scene(&mut scene, &theme);

        {
            let mut group = scene.get_view_mut(&ViewId::new("listview")).unwrap();
            let state = &mut group.get_state::<SelectOneOfState>().unwrap();
            assert_eq!(state.selected, 0);
        }

        click_at(&mut scene, &vec![], Point::new(100, 10));

        {
            let state = &mut scene
                .get_view_state::<SelectOneOfState>(&ViewId::new("listview"))
                .unwrap();
            assert_eq!(state.selected, 1);
        }

        let mut ctx = MockDrawingContext::new(&scene);
        assert_eq!(scene.dirty, true);
        draw_scene(&mut scene, &mut ctx, &theme);
        assert_eq!(scene.dirty, false);
    }
}
