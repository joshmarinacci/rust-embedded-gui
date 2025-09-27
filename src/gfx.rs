use crate::geom::{Bounds, Point};
use embedded_graphics::Drawable;
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::text::Text;
use crate::view::Align;

pub struct TextStyle<'a> {
    pub halign: Align,
    pub valign: Align,
    pub underline: bool,
    pub font: &'a MonoFont<'static>,
    pub color: &'a Rgb565,
}

impl<'a> TextStyle<'a> {
    pub fn new(font: &'a MonoFont<'static>, color: &'a Rgb565) -> TextStyle<'a> {
        TextStyle {
            font,
            color,
            underline: false,
            valign: Align::Center,
            halign: Align::Start,
        }
    }
    pub fn with_underline(&self, underline: bool) -> Self {
        TextStyle {
            color: self.color,
            font: self.font,
            underline,
            halign: self.halign,
            valign: self.valign,
        }
    }
    pub fn with_halign(&self, halign: Align) -> Self {
        TextStyle {
            color: self.color,
            font: self.font,
            underline: self.underline,
            halign,
            valign: self.valign,
        }
    }
}

pub trait DrawingContext {
    fn fill_rect(&mut self, bounds: &Bounds, color: &Rgb565);
    fn stroke_rect(&mut self, bounds: &Bounds, color: &Rgb565);
    fn line(&mut self, start: &Point, end: &Point, color: &Rgb565);
    fn fill_text(&mut self, bounds: &Bounds, text: &str, style: &TextStyle);
    fn text(&mut self, text: &str, position: &Point, style: &TextStyle);
    fn translate(&mut self, offset: &Point);
}

pub fn draw_centered_text(
    ctx: &mut dyn DrawingContext,
    text: &str,
    bounds: &Bounds,
    font: &MonoFont<'static>,
    color: &Rgb565,
) {
    ctx.text(
        text,
        &bounds.center(),
        &TextStyle {
            font,
            color,
            valign: Align::Center,
            halign: Align::Center,
            underline: false,
        },
    )
}
