use crate::gfx::{TextStyle};
use crate::util;
use crate::view::{View, ViewId};
use crate::view::Flex::Intrinsic;

pub fn make_label(name: &'static str, title: &str) -> View {
    View {
        name: ViewId::new(name),
        title: title.into(),
        h_flex: Intrinsic,
        v_flex: Intrinsic,
        layout: Some(|e| {
            if let Some(view) = e.scene.get_view_mut(e.target) {
                view.bounds.size = util::calc_size(e.theme.bold_font, &view.title);
            }
        }),
        draw: Some(|e| {
            let style = TextStyle::new(&e.theme.font, &e.theme.fg);
            e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
        }),
        .. Default::default()
    }
}
