use embedded_graphics::Drawable;
use embedded_graphics::mock_display::MockDisplay;
use embedded_graphics::mono_font::ascii::FONT_7X13_BOLD;
use embedded_graphics::mono_font::iso_8859_9::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor, WebColors};
use embedded_graphics::primitives::{Line, Primitive, PrimitiveStyle};
use embedded_graphics::geometry::{Point as EPoint};
use embedded_graphics::text::Text;
use crate::geom::{Bounds, Point};
use crate::gfx::{DrawingContext, TextStyle};
use crate::scene::Scene;
use crate::{util, Theme};

pub struct MockDrawingContext {
    pub clip_rect: Bounds,
    pub display: MockDisplay<Rgb565>,
    offset: Point,
}

impl MockDrawingContext {
    pub fn new(scene: &Scene) -> MockDrawingContext {
        let mut ctx: MockDrawingContext = MockDrawingContext {
            clip_rect: scene.dirty_rect,
            display: MockDisplay::new(),
            offset: Point::new(0, 0),
        };
        ctx.display.set_allow_out_of_bounds_drawing(true);
        ctx.display.set_allow_overdraw(true);
        return ctx;
    }
    pub fn make_mock_theme() -> Theme {
        Theme {
            bg: Rgb565::WHITE,
            fg: Rgb565::BLACK,
            selected_bg: Rgb565::WHITE,
            selected_fg: Rgb565::BLACK,
            panel_bg: Rgb565::CSS_GRAY,
            font: FONT_6X10,
            bold_font: FONT_7X13_BOLD,
        }
    }
}
impl DrawingContext for MockDrawingContext {
    fn fill_rect(&mut self, bounds: &Bounds, color: &Rgb565) {
        // info!("fill_rect {:?} {:?} {:?}", bounds, self.clip_rect, color);
        util::bounds_to_rect(bounds)
            .intersection(&util::bounds_to_rect(&self.clip_rect))
            .into_styled(PrimitiveStyle::with_fill(*color))
            .draw(&mut self.display)
            .unwrap();
    }

    fn stroke_rect(&mut self, bounds: &Bounds, color: &Rgb565) {
        util::bounds_to_rect(bounds)
            .intersection(&util::bounds_to_rect(&self.clip_rect))
            .into_styled(PrimitiveStyle::with_stroke(*color, 1))
            .draw(&mut self.display)
            .unwrap();
    }

    fn line(&mut self, start: &Point, end: &Point, color: &Rgb565) {
        let line = Line::new(EPoint::new(start.x,start.y),EPoint::new(end.x,end.y));
        line.into_styled(PrimitiveStyle::with_stroke(*color,1)).draw(&mut self.display).unwrap();
    }

    // fn fill_text(&mut self, bounds: &Bounds, text: &str, style: &TextStyle);
    fn fill_text(&mut self, bounds: &Bounds, text: &str, style: &TextStyle) {
        let style = MonoTextStyle::new(&style.font, *style.color);
        let mut pt = embedded_graphics::geometry::Point::new(bounds.x, bounds.y);
        pt.y += bounds.h / 2;
        pt.y += (style.font.baseline as i32) / 2;
        let w = (style.font.character_size.width as i32) * (text.len() as i32);
        pt.x += (bounds.w - w) / 2;
        Text::new(text, pt, style).draw(&mut self.display).unwrap();
    }

    fn text(&mut self, text: &str, position: &Point, style: &TextStyle) {

    }

    fn translate(&mut self, offset: &Point) {
        self.offset = self.offset.add(offset);
    }
}