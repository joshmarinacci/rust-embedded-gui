use crate::geom::Bounds;
use crate::view::View;
use crate::{DrawingContext, LayoutEvent};

pub fn make_panel(name: &str, bounds: Bounds) -> View {
    View {
        name: name.into(),
        title: name.into(),
        bounds,
        visible: true,
        input: None,
        state: None,
        layout: None,
        draw: Some(|e| {
            e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
            e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
        }),
    }
}

pub fn layout_vbox(evt: &mut LayoutEvent) {
    if let Some(parent) = evt.scene.get_view_mut(evt.target) {
        let bounds = parent.bounds;
        let mut y = bounds.y;
        for kid in evt.scene.get_children(evt.target) {
            if let Some(ch) = evt.scene.get_view_mut(&kid) {
                ch.bounds.x = bounds.x;
                ch.bounds.y = y;
                y += ch.bounds.h;
            }
        }
    }
}

pub fn layout_hbox(evt: &mut LayoutEvent) {
    if let Some(parent) = evt.scene.get_view_mut(evt.target) {
        let bounds = parent.bounds;
        let mut x = bounds.x;
        for kid in evt.scene.get_children(evt.target) {
            if let Some(ch) = evt.scene.get_view_mut(&kid) {
                ch.bounds.x = x;
                ch.bounds.y = bounds.y;
                x += ch.bounds.w;
            }
        }
    }
}
