use crate::DrawEvent;
use core::ops::Sub;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;

pub fn draw_std_panel(e: &mut DrawEvent) {
    let bounds = e.view.bounds;
    e.ctx.fill_rect(&bounds, &e.theme.bg);
    e.ctx.stroke_rect(&bounds, &e.theme.fg);
    e.ctx.stroke_rect(&bounds.sub(e.view.padding), &Rgb565::RED);
}
