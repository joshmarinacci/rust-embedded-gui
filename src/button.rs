use crate::gfx::draw_centered_text;
use crate::view::Flex::Intrinsic;
use crate::view::{View, ViewId};
use crate::{Action, DrawEvent, EventType, util};
use alloc::string::ToString;

fn draw_button(e: &mut DrawEvent) {
    e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
    if let Some(focused) = e.focused {
        if focused == &e.view.name {
            e.ctx.stroke_rect(&e.view.bounds.contract(2), &e.theme.fg);
        }
    }
    draw_centered_text(
        e.ctx,
        &e.view.title,
        &e.view.bounds,
        &e.theme.bold_font,
        &e.theme.fg,
    );
}

pub fn make_button(name: &'static str, title: &str) -> View {
    View {
        name: ViewId::new(name),
        title: title.to_string(),
        h_flex: Intrinsic,
        v_flex: Intrinsic,
        input: Some(|e| {
            if let EventType::Tap(_pt) = &e.event_type {
                e.scene.set_focused(e.target);
                e.scene.mark_dirty_view(e.target);
                return Some(Action::Generic);
            }
            None
        }),
        layout: Some(|e| {
            if let Some(view) = e.scene.get_view_mut(&e.target) {
                view.bounds.size = util::calc_size(e.theme.bold_font, &view.title);
            }
        }),
        draw: Some(draw_button),
        ..Default::default()
    }
}
