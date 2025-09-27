use crate::geom::Bounds;
use crate::gfx::{DrawingContext, TextStyle};
use crate::util;
use crate::view::View;

pub fn make_label(name: &str, title: &str) -> View {
    View {
        name: name.into(),
        title: title.into(),
        bounds: Bounds::new(0,0,100,30),
        visible: true,
        state: None,
        input: None,
        draw: Some(|e| {
            let style = TextStyle::new(&e.theme.font, &e.theme.fg);
            e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
        }),
        layout: Some(|e| {
            if let Some(view) = e.scene.get_view_mut(e.target) {
                view.bounds = util::calc_bounds(view.bounds, e.theme.bold_font, &view.title);
            }
        }),
        .. Default::default()
    }
}
