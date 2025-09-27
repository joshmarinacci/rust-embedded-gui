use crate::LayoutEvent;
use crate::geom::Bounds;
use crate::gfx::{DrawingContext, HAlign, VAlign};
use crate::view::View;
use alloc::boxed::Box;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use log::info;

pub struct PanelState {
    pub padding: i32,
    pub gap: i32,
    pub debug: bool,
    pub border: bool,
    pub bg: bool,
    pub halign: HAlign,
    pub valign: VAlign,
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
            bg: true,
            gap: 0,
            halign: HAlign::Center,
            valign: VAlign::Center,
        })),
        layout: None,
        draw: Some(|e| {
            let bounds = e.view.bounds;
            if let Some(state) = e.view.get_state::<PanelState>() {
                if state.bg {
                    e.ctx.fill_rect(&bounds, &e.theme.bg);
                }
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
    if let Some(view) = evt.scene.get_view(evt.target) {
        let bounds = view.bounds;
        if let Some(state) = evt.scene.get_view_state::<PanelState>(evt.target) {
            let padding = state.padding;
            let gap = state.gap;
            let mut y = padding + 0;
            let halign = state.halign;
            for kid in evt.scene.get_children(evt.target) {
                if let Some(ch) = evt.scene.get_view_mut(&kid) {
                    match halign {
                        HAlign::Left => ch.bounds.position.x = padding,
                        HAlign::Center => {
                            ch.bounds.position.x = padding + (bounds.w() - padding * 2 - ch.bounds.w()) / 2
                        }
                        HAlign::Right => ch.bounds.position.x = bounds.w() - padding - ch.bounds.w(),
                    }
                    ch.bounds.position.y = y;
                    y += ch.bounds.size.h + gap;
                }
            }
        }
    }
}

pub fn layout_hbox(evt: &mut LayoutEvent) {
    if let Some(view) = evt.scene.get_view(evt.target) {
        let height = view.bounds.h();
        if let Some(state) = evt.scene.get_view_state::<PanelState>(evt.target) {
            let padding = state.padding;
            let gap = state.gap;
            let mut x = padding;
            let valign = state.valign;
            for kid in evt.scene.get_children(evt.target) {
                if let Some(ch) = evt.scene.get_view_mut(&kid) {
                    match valign {
                        VAlign::Top => ch.bounds.position.y= padding,
                        VAlign::Center => {
                            ch.bounds.position.y= padding + (height - padding * 2 - ch.bounds.h()) / 2
                        }
                        VAlign::Bottom => ch.bounds.position.y= height - padding - ch.bounds.h(),
                    }
                    ch.bounds.position.x= x;
                    // ch.bounds.position.y= padding;
                    x += ch.bounds.size.w+ gap;
                }
            }
        }
    }
}
