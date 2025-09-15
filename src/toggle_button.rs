use crate::geom::Bounds;
use crate::{Action, DrawingContext, GuiEvent, TextStyle, Theme};
use alloc::boxed::Box;
use core::any::Any;
use core::option::Option::*;
use crate::view::View;

pub fn make_toggle_button<C, F>(name: &str, title: &str) -> View<C, F> {
    View {
        name: name.into(),
        title: title.into(),
        bounds: Bounds::new(0, 0, 80, 30),
        visible: true,
        state: Some(Box::new(SelectedState::new())),
        draw: Some(draw_toggle_button),
        draw2: None,
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

fn draw_toggle_button<C, F>(
    view: &mut View<C, F>,
    ctx: &mut dyn DrawingContext<C, F>,
    theme: &Theme<C, F>,
) {
    let (button_fill, button_color) = if let Some(state) = view.get_state::<SelectedState>() {
        if state.selected {
            (&theme.fg, &theme.bg)
        } else {
            (&theme.bg, &theme.fg)
        }
    } else {
        (&theme.bg, &theme.fg)
    };

    ctx.fill_rect(&view.bounds, button_fill);
    ctx.stroke_rect(&view.bounds, &theme.fg);
    let style = TextStyle::new(&theme.font, button_color);
    ctx.fill_text(&view.bounds, &view.title, &style);
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
    use crate::toggle_button::{make_toggle_button, SelectedState};
    use crate::{click_at, draw_scene, layout_scene, MockDrawingContext, Theme};
    use alloc::string::String;
    use alloc::vec;
    use crate::scene::Scene;

    #[test]
    fn test_toggle_button() {
        let theme: Theme<String, String> = Theme {
            bg: "white".into(),
            fg: "black".into(),
            panel_bg: "grey".into(),
            font: "plain".into(),
            bold_font: "bold".into(),
        };
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
}
