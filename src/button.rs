use crate::geom::Bounds;
use crate::gfx::{DrawingContext, HAlign, TextStyle, VAlign, draw_centered_text};
use crate::view::View;
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

pub fn make_button(name: &str, title: &str) -> View {
    View {
        name: name.to_string(),
        title: title.to_string(),
        bounds: Bounds::new(0,0,80,30),
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
                view.bounds = util::calc_bounds(view.bounds, e.theme.bold_font, &view.title);
            }
        }),
        draw: Some(draw_button),
        .. Default::default()
    }
}
