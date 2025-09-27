use crate::geom::Bounds;
use embedded_graphics::geometry::Size;
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::primitives::Rectangle;
use log::info;

pub fn calc_bounds(bounds: Bounds, font: MonoFont<'static>, title: &str) -> Bounds {
    let fsize = &font.character_size;
    let hpad = fsize.width;
    let vpad = fsize.height / 2;
    let mut width = fsize.width * title.len() as u32;
    width += hpad * 2;
    let mut height = fsize.height;
    height += vpad * 2;
    Bounds::new(bounds.x(), bounds.y(), width as i32, height as i32)
}

pub fn bounds_to_rect(bounds: &Bounds) -> Rectangle {
    if bounds.is_empty() {
        return Rectangle::zero();
    }
    Rectangle::new(
        embedded_graphics::geometry::Point::new(bounds.position.x, bounds.position.y),
        Size::new(bounds.size.w as u32, bounds.size.h as u32),
    )
}
