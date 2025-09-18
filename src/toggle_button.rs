use crate::geom::Bounds;
use crate::view::View;
use crate::{Action, DrawEvent, DrawingContext, GuiEvent, TextStyle, Theme};
use alloc::boxed::Box;
use core::any::Any;
use core::option::Option::*;

pub fn make_toggle_button<C, F>(name: &str, title: &str) -> View<C, F> {
    View {
        name: name.into(),
        title: title.into(),
        bounds: Bounds::new(0, 0, 80, 30),
        visible: true,
        state: Some(Box::new(SelectedState::new())),
        draw: Some(draw_toggle_button),
        layout: None,
        input: Some(input_toggle_button),
    }
}

pub struct SelectedState {
    pub selected: bool,
}

impl SelectedState {
    pub fn new() -> SelectedState {
        SelectedState { selected: false }
    }
}

fn draw_toggle_button<C, F>(e: &mut DrawEvent<C, F>) {
    let (button_fill, button_color) = if let Some(state) = e.view.get_state::<SelectedState>() {
        if state.selected {
            (&e.theme.fg, &e.theme.bg)
        } else {
            (&e.theme.bg, &e.theme.fg)
        }
    } else {
        (&e.theme.bg, &e.theme.fg)
    };

    e.ctx.fill_rect(&e.view.bounds, button_fill);
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
    let style = TextStyle::new(&e.theme.font, button_color);
    e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
}

fn input_toggle_button<C, F>(event: &mut GuiEvent<C, F>) -> Option<Action> {
    if let Some(state) = event.scene.get_view_state::<SelectedState>(event.target) {
        state.selected = !state.selected;
    }
    event.scene.mark_dirty_view(event.target);
    None
}

mod tests {
    use crate::geom::{Bounds, Point};
    use crate::scene::{Scene, click_at, draw_scene, layout_scene};
    use crate::toggle_button::{SelectedState, make_toggle_button};
    use crate::{MockDrawingContext, Theme};
    use alloc::string::String;
    use alloc::vec;

    #[test]
    fn test_toggle_button() {
        let theme = make_mock_theme();
        let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));
        {
            let mut button = make_toggle_button("toggle", "Toggle");
            scene.add_view_to_root(button);
        }
        layout_scene(&mut scene);

        {
            let mut button = scene.get_view_mut("toggle").unwrap();
            assert_eq!(button.name, "toggle");
            assert_eq!(button.bounds, Bounds::new(0, 0, 80, 30));
            let state = &mut button.get_state::<SelectedState>().unwrap();
            assert_eq!(state.selected, false);
        }

        click_at(&mut scene, &vec![], Point::new(10, 10));

        {
            let state = scene.get_view_state::<SelectedState>("toggle").unwrap();
            assert_eq!(state.selected, true);
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

    fn make_mock_theme() -> Theme<String, String> {
        Theme {
            bg: "white".into(),
            fg: "black".into(),
            panel_bg: "grey".into(),
            font: "plain".into(),
            bold_font: "bold".into(),
        }
    }
}
