use crate::gfx::draw_centered_text;
use crate::input::{InputEvent, OutputAction};
use crate::view::Flex::Intrinsic;
use crate::view::{View, ViewId};
use crate::util;
use alloc::boxed::Box;
use alloc::string::{String, ToString};

pub struct ButtonState {
    command: String,
    primary: bool,
}

pub fn make_button(name: &ViewId, title: &str) -> View {
    make_full_button(name, title, title.into(), false)
}
pub fn make_full_button(name: &ViewId, title: &str, command: String, primary: bool) -> View {
    View {
        name: name.clone(),
        title: title.to_string(),
        h_flex: Intrinsic,
        v_flex: Intrinsic,
        state: Some(Box::new(ButtonState {
            primary,
            command,
        })),
        input: Some(|e| {
            if let InputEvent::Tap(_pt) = &e.event_type {
                e.scene.set_focused(e.target);
                if let Some(state) = e.scene.get_view_state::<ButtonState>(e.target) {
                    return Some(OutputAction::Command(state.command.clone()));
                }
            }
            None
        }),
        layout: Some(|e| {
            if let Some(view) = e.scene.get_view_mut(&e.target) {
                view.bounds.size = util::calc_size(e.theme.bold_font, &view.title);
            }
        }),
        draw: Some(|e| {
            let Some(state) = e.view.get_state::<ButtonState>() else {
                return
            };
            if state.primary {
                e.ctx.fill_rect(&e.view.bounds, &e.theme.accented.fill);
                e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
                if e.focused == &Some(e.view.name) {
                    e.ctx.stroke_rect(&e.view.bounds.contract(2), &e.theme.accented.text);
                }
                draw_centered_text(
                    e.ctx,
                    &e.view.title,
                    &e.view.bounds,
                    &e.theme.bold_font,
                    &e.theme.accented.text,
                );
            } else {
                e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
                e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
                if e.focused == &Some(e.view.name) {
                    e.ctx.stroke_rect(&e.view.bounds.contract(2), &e.theme.fg);
                }
                draw_centered_text(
                    e.ctx,
                    &e.view.title,
                    &e.view.bounds,
                    &e.theme.bold_font,
                    &e.theme.fg,
                );
            }
        }),
        ..Default::default()
    }
}
