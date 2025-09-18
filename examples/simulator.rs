#[cfg(feature = "std")]
use embedded_graphics::{
    prelude::*,
    primitives::{
        Circle, PrimitiveStyleBuilder, StrokeAlignment, Triangle,
    },
};
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::ascii::{FONT_6X10, FONT_7X13_BOLD};
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use log::error;
use gui2::comps::{make_button, make_label, make_panel, make_text_input};
use gui2::geom::Bounds;
use gui2::scene::{draw_scene, Scene};
use gui2::{DrawingContext, TextStyle, Theme};

fn make_gui_scene() -> Scene<Rgb565, MonoFont<'static>> {
    let mut scene: Scene<Rgb565, MonoFont> = Scene::new_with_bounds(Bounds::new(0,0,320,240));

    let mut panel = make_panel("panel",Bounds{x:20,y:20,w:200,h:200});

    scene.add_view_to_parent(make_label("label1","A Label").position_at(10,30),
                             &panel.name);

    scene.add_view_to_root(make_button("button1","A button")
        .position_at(10,60));
    scene.add_view_to_root(make_button("button2","A button")
        .position_at(10,120));

    scene.add_view_to_root(make_button("button3","A button")
        .position_at(10,200));
    scene.mark_dirty_all();

    scene.add_view_to_parent(
        make_text_input("textinput","type text here")
            .position_at(10,90),&panel.name);

    scene.add_view_to_root(panel);

    scene
}
struct SimulatorDrawingContext {
    pub clip_rect: Bounds,
    display: SimulatorDisplay<Rgb565>,
}

impl SimulatorDrawingContext {
    fn new(display: SimulatorDisplay<Rgb565>) -> SimulatorDrawingContext {
        SimulatorDrawingContext {
            display,
            clip_rect: Bounds::new_empty(),
        }
    }
}

fn bounds_to_rect(bounds: &Bounds) -> Rectangle {
    Rectangle::new(Point::new(bounds.x,bounds.y),
                   Size::new(bounds.w as u32,bounds.h as u32))
}

impl DrawingContext<Rgb565, MonoFont<'static>> for SimulatorDrawingContext {
    fn clear(&mut self, color: &Rgb565) {
        error!("clear {:?}", color);
        self.display.clear(*color).unwrap();
    }

    fn fill_rect(&mut self, bounds: &Bounds, color: &Rgb565) {
        // info!("fill_rect {:?} {:?} {:?}", bounds, self.clip_rect, color);
        bounds_to_rect(bounds)
            .intersection(&bounds_to_rect(&self.clip_rect))
            .into_styled(PrimitiveStyle::with_fill(*color))
            .draw(&mut self.display).unwrap();

    }

    fn stroke_rect(&mut self, bounds: &Bounds, color: &Rgb565) {
        bounds_to_rect(bounds)
            .intersection(&bounds_to_rect(&self.clip_rect))
            .into_styled(PrimitiveStyle::with_stroke(*color,1))
            .draw(&mut self.display).unwrap();
    }

    // fn fill_text(&mut self, bounds: &Bounds, text: &str, style: &TextStyle<C, F>);
    fn fill_text(&mut self, bounds: &Bounds, text: &str, style: &TextStyle<Rgb565, MonoFont<'static>>) {
        let style = MonoTextStyle::new(&style.font, *style.color);
        let mut pt = Point::new(bounds.x, bounds.y);
        pt.y += bounds.h / 2;
        pt.y += (style.font.baseline as i32)/2;
        let w = (style.font.character_size.width as i32) * (text.len() as i32);
        pt.x += (bounds.w - w) / 2;
        Text::new(text, pt, style)
            .draw(&mut self.display)
            .unwrap();
    }
}

fn main() -> Result<(), std::convert::Infallible> {
    // Create a new simulator display with 128x64 pixels.
    let display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));
    let mut ctx:SimulatorDrawingContext = SimulatorDrawingContext::new(display);

    let mut scene = make_gui_scene();
    let theme:Theme<Rgb565, MonoFont> = Theme {
        bg: Rgb565::WHITE,
        fg: Rgb565::BLACK,
        panel_bg: Rgb565::CSS_LIGHT_GRAY,
        font: FONT_6X10,
        bold_font: FONT_7X13_BOLD,
    };
    ctx.clip_rect = scene.dirty_rect.clone();
    draw_scene(&mut scene, &mut ctx, &theme);



    let output_settings = OutputSettingsBuilder::new()
        .scale(2)
        // .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Simulator Test", &output_settings).show_static(&ctx.display);
    Ok(())
}