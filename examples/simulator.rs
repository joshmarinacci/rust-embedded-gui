use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::ascii::{
    FONT_6X10, FONT_7X13_BOLD, FONT_9X15, FONT_9X15_BOLD,
};
use embedded_graphics::mono_font::iso_8859_9::FONT_7X13;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
#[cfg(feature = "std")]
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use gui2::comps::{make_button, make_label, make_panel, make_text_input};
use gui2::geom::{Bounds, Point as GPoint};
use gui2::scene::{click_at, draw_scene, layout_scene, EventResult, Scene};
use gui2::toggle_button::make_toggle_button;
use gui2::{DrawingContext, TextStyle, Theme};
use gui2::toggle_group::make_toggle_group;

const SMALL_FONT_BUTTON: &str = "small_font";
const MEDIUM_FONT_BUTTON: &str = "medium_font";
const LARGE_FONT_BUTTON: &str = "large_font";

fn make_scene() -> Scene {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

    let panel = make_panel(
        "panel",
        Bounds {
            x: 20,
            y: 20,
            w: 320 - 40,
            h: 160,
        },
    );

    scene.add_view_to_parent(
        make_label("label1", "A Label").position_at(30, 30),
        &panel.name,
    );
    scene.add_view_to_parent(
        make_toggle_button("toggle1", "Toggle Me").position_at(30, 60),
        &panel.name,
    );
    scene.add_view_to_parent(
        make_text_input("textinput", "type text here").position_at(30, 90),
        &panel.name,
    );
    scene.add_view_to_parent(
        make_toggle_group("toggle2",vec!["Apple","Ball","Car"],1).position_at(30, 130),
        &panel.name
    );

    scene.add_view_to_root(panel);

    scene.add_view_to_root(make_button(SMALL_FONT_BUTTON, "Small").position_at(30, 200));
    scene.add_view_to_root(make_button(MEDIUM_FONT_BUTTON, "Medium").position_at(120, 200));
    scene.add_view_to_root(make_button(LARGE_FONT_BUTTON, "Large").position_at(220, 200));



    scene
}
struct SimulatorDrawingContext<'a> {
    pub clip_rect: Bounds,
    display: &'a mut SimulatorDisplay<Rgb565>,
}

impl SimulatorDrawingContext<'_> {
    fn new(display: &mut SimulatorDisplay<Rgb565>) -> SimulatorDrawingContext {
        SimulatorDrawingContext {
            display,
            clip_rect: Bounds::new_empty(),
        }
    }
}

fn bounds_to_rect(bounds: &Bounds) -> Rectangle {
    Rectangle::new(
        Point::new(bounds.x, bounds.y),
        Size::new(bounds.w as u32, bounds.h as u32),
    )
}

impl DrawingContext for SimulatorDrawingContext<'_> {
    fn fill_rect(&mut self, bounds: &Bounds, color: &Rgb565) {
        // info!("fill_rect {:?} {:?} {:?}", bounds, self.clip_rect, color);
        bounds_to_rect(bounds)
            .intersection(&bounds_to_rect(&self.clip_rect))
            .into_styled(PrimitiveStyle::with_fill(*color))
            .draw(self.display)
            .unwrap();
    }
    fn stroke_rect(&mut self, bounds: &Bounds, color: &Rgb565) {
        bounds_to_rect(bounds)
            .intersection(&bounds_to_rect(&self.clip_rect))
            .into_styled(PrimitiveStyle::with_stroke(*color, 1))
            .draw(self.display)
            .unwrap();
    }
    fn fill_text(&mut self, bounds: &Bounds, text: &str, style: &TextStyle) {
        let style = MonoTextStyle::new(&style.font, *style.color);
        let mut pt = Point::new(bounds.x, bounds.y);
        pt.y += bounds.h / 2;
        pt.y += (style.font.baseline as i32) / 2;
        let w = (style.font.character_size.width as i32) * (text.len() as i32);
        pt.x += (bounds.w - w) / 2;
        Text::new(text, pt, style).draw(self.display).unwrap();
    }
}

fn main() -> Result<(), std::convert::Infallible> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));

    let mut scene = make_scene();
    let mut theme = Theme {
        bg: Rgb565::WHITE,
        fg: Rgb565::BLACK,
        panel_bg: Rgb565::CSS_LIGHT_GRAY,
        font: FONT_7X13,
        bold_font: FONT_7X13_BOLD,
    };

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Simulator Test", &output_settings);
    'running: loop {
        let mut ctx: SimulatorDrawingContext = SimulatorDrawingContext::new(&mut display);
        ctx.clip_rect = scene.dirty_rect.clone();
        layout_scene(&mut scene, &theme);
        draw_scene(&mut scene, &mut ctx, &theme);
        window.update(&display);
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => {}
                SimulatorEvent::MouseButtonUp { point, .. } => {
                    println!("mouse button up {}", point);
                    if let Some(result) =
                        click_at(&mut scene, &vec![], GPoint::new(point.x, point.y))
                    {
                        handle_events(result, &mut scene, &mut theme);
                    }
                }
                SimulatorEvent::MouseButtonDown { mouse_btn, point } => {
                    println!("mouse down");
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn handle_events(result: EventResult, scene: &mut Scene, theme: &mut Theme) {
    let (name, action) = result;
    println!("result of event {:?} from {name}", action);
    if name == SMALL_FONT_BUTTON {
        theme.font = FONT_6X10;
        theme.bold_font = FONT_6X10;
        scene.mark_layout_dirty();
        scene.mark_dirty_all();
    }
    if name == MEDIUM_FONT_BUTTON {
        theme.font = FONT_7X13;
        theme.bold_font = FONT_7X13_BOLD;
        scene.mark_layout_dirty();
        scene.mark_dirty_all();
    }
    if name == LARGE_FONT_BUTTON {
        theme.font = FONT_9X15;
        theme.bold_font = FONT_9X15_BOLD;
        scene.mark_layout_dirty();
        scene.mark_dirty_all();
    }
}
