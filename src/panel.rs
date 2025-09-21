use crate::LayoutEvent;
use crate::geom::Bounds;
use crate::gfx::DrawingContext;
use crate::view::View;
use alloc::boxed::Box;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};

pub struct PanelState {
    pub padding: i32,
    pub gap: i32,
    pub debug: bool,
    pub border: bool,
}
pub fn make_panel(name: &str, bounds: Bounds) -> View {
    View {
        name: name.into(),
        title: name.into(),
        bounds,
        visible: true,
        input: None,
        state: Some(Box::new(PanelState {
            padding: 0,
            debug: false,
            border: true,
            gap: 0,
        })),
        layout: None,
        draw: Some(|e| {
            e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
            let bounds = e.view.bounds;
            if let Some(state) = e.view.get_state::<PanelState>() {
                if state.border {
                    e.ctx.stroke_rect(&bounds, &e.theme.fg);
                }
                if state.debug {
                    let bds = bounds.contract(state.padding);
                    e.ctx.stroke_rect(&bds, &Rgb565::RED);
                }
            }
        }),
    }
}

pub fn layout_vbox(evt: &mut LayoutEvent) {
    if let Some(state) = evt.scene.get_view_state::<PanelState>(evt.target) {
        let padding = state.padding;
        let gap = state.gap;
        let mut y = padding;
        for kid in evt.scene.get_children(evt.target) {
            if let Some(ch) = evt.scene.get_view_mut(&kid) {
                ch.bounds.x = padding;
                ch.bounds.y = y;
                y += ch.bounds.h + gap;
            }
        }
    }
}

pub fn layout_hbox(evt: &mut LayoutEvent) {
    if let Some(state) = evt.scene.get_view_state::<PanelState>(evt.target) {
        let padding = state.padding;
        let gap = state.gap;
        let mut x = padding;
        for kid in evt.scene.get_children(evt.target) {
            if let Some(ch) = evt.scene.get_view_mut(&kid) {
                ch.bounds.x = x;
                ch.bounds.y = padding;
                x += ch.bounds.w + gap;
            }
        }
    }
}
