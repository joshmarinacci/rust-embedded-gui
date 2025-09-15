#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use embedded_hal_bus::spi::{ExclusiveDevice, RefCellDevice};
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::Level::{High, Low};
use esp_hal::gpio::{Input, InputConfig, Output, OutputConfig, Pull};
use esp_hal::{main, Blocking};
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::{Duration, Instant, Rate};
use log::{error, info};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::ascii::{FONT_7X13, FONT_7X13_BOLD, FONT_9X15};
use embedded_graphics::mono_font::MonoFont;
use esp_hal::i2c::master::{BusTimeout, Config as I2CConfig, I2c};
use mipidsi::interface::SpiInterface;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::{models::ST7789, Builder, Display, NoResetPin};
use static_cell::StaticCell;

use gui2::geom::{Bounds, Point as GPoint};
use gt911::Gt911Blocking;
use gui2::comps::{make_button, make_label, make_panel, make_text_input};
use gui2::scene::{draw_scene, pick_at, Scene};
use gui2::{Action, DrawingContext, EventType, GuiEvent, HAlign, LayoutEvent, TextStyle, Theme};
use gui2::view::View;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

extern crate alloc;

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let mut delay = Delay::new();

    // have to turn on the board and wait 500ms before using the keyboard
    let mut board_power = Output::new(peripherals.GPIO10, High, OutputConfig::default());
    board_power.set_high();
    delay.delay_millis(1000);

    // ==== display setup ====
    // https://github.com/Xinyuan-LilyGO/T-Deck/blob/master/examples/HelloWorld/HelloWorld.ino

    let mut TFT_CS = Output::new(peripherals.GPIO12, High,
                                 OutputConfig::default());
    TFT_CS.set_high();
    let tft_dc = Output::new(peripherals.GPIO11, Low,
                             OutputConfig::default());
    let mut tft_enable = Output::new(peripherals.GPIO42, High,
                                     OutputConfig::default());
    tft_enable.set_high();

    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default().with_frequency(Rate::from_mhz(40)),
    ).unwrap()
        .with_sck(peripherals.GPIO40)
        .with_miso(Input::new(peripherals.GPIO38,
                              InputConfig::default().with_pull(Pull::Up)))
        .with_mosi(peripherals.GPIO41);

    static DISPLAY_BUF: StaticCell<[u8; 512]> = StaticCell::new();
    let buffer = DISPLAY_BUF.init([0u8; 512]);

    info!("setting up the display");
    let spi_delay = Delay::new();
    let spi_device = ExclusiveDevice::new(spi, TFT_CS, spi_delay).unwrap();
    let di = SpiInterface::new(spi_device, tft_dc, buffer);
    info!("building");
    let display = Builder::new(ST7789, di)
        .display_size(240, 320)
        .invert_colors(ColorInversion::Inverted)
        .color_order(ColorOrder::Rgb)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .init(&mut delay)
        .unwrap();

    info!("initialized display");
    // wait for everything to boot up
    // delay.delay_millis(500);
    info!("Display initialized");

    let mut ctx:EmbeddedDrawingContext = EmbeddedDrawingContext::new(display);
    let mut scene: Scene<Rgb565, MonoFont> = make_gui_scene();


    let theme:Theme<Rgb565, MonoFont> = Theme {
        bg: Rgb565::WHITE,
        fg: Rgb565::BLACK,
        panel_bg: Rgb565::CSS_LIGHT_GRAY,
        font: FONT_6X10,
        bold_font: FONT_7X13_BOLD,
    };

    static I2C: StaticCell<I2c<Blocking>> = StaticCell::new();

    let i2c = I2c::new(
        peripherals.I2C0,
        I2CConfig::default()
            .with_frequency(Rate::from_khz(100))
            .with_timeout(BusTimeout::Disabled),
    )
        .unwrap()
        .with_sda(peripherals.GPIO18)
        .with_scl(peripherals.GPIO8);
    info!("initialized I2C keyboard");
    let i2c_ref = I2C.init(i2c);

    let touch = Gt911Blocking::default();
    touch.init(i2c_ref).unwrap();

    loop {
        // info!("checking input");
        if let Ok(point) = touch.get_touch(i2c_ref) {
            if let Some(point) = point {
                // flip because the screen is mounted sideways on the t-deck
                let pt = GPoint::new(320 - point.y as i32, 240-point.x as i32);
                let targets = pick_at(&mut scene, &pt);
                info!("clicked on targets {:?}", targets);
                if let Some(target) =  targets.last() {
                    let mut evt:GuiEvent<Rgb565, MonoFont> = GuiEvent {
                        scene: &mut scene,
                        target,
                        event_type: EventType::Tap(pt),
                        action: None,
                    };
                    info!("created event on target {:?} at {:?}",evt.target, evt.event_type);
                    if let Some(view) = evt.scene.get_view(evt.target) {
                        if let Some(input) = view.input {
                            input(&mut evt);
                        }
                    }
                }
            }
        }

        let delay_start = Instant::now();
        ctx.clip_rect = scene.dirty_rect.clone();
        draw_scene(&mut scene, &mut ctx, &theme);
        while delay_start.elapsed() < Duration::from_millis(100) {}
    }
}


fn make_gui_scene() -> Scene<Rgb565, MonoFont<'static>> {
    let mut scene: Scene<Rgb565, MonoFont> = Scene::new_with_bounds(Bounds::new(0,0,320,240));
    let rootname = scene.root_id.clone();

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

    scene.add_view_to_parent(make_menuview("menuview",vec!["first".into(),"second".into(),"third".into()])
                                 .position_at(100,30), &panel.name);
    scene.add_view_to_root(panel);

    scene
}

struct EmbeddedDrawingContext {
    pub display: Display<
        SpiInterface<
            'static,
            ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, Delay>,
            Output<'static>,
        >,
        ST7789,
        NoResetPin,
    >,
    pub clip_rect: Bounds,
}

impl EmbeddedDrawingContext {
    fn new(display: Display<
        SpiInterface<
            'static,
            ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, Delay>,
            Output<'static>>,
        ST7789,
        NoResetPin
    >) -> EmbeddedDrawingContext {
        EmbeddedDrawingContext {
            display,
            clip_rect: Bounds::new_empty(),
        }
    }
}

fn bounds_to_rect(bounds: &Bounds) -> Rectangle {
    Rectangle::new(Point::new(bounds.x,bounds.y),
                   Size::new(bounds.w as u32,bounds.h as u32))
}

impl DrawingContext<Rgb565, MonoFont<'static>> for EmbeddedDrawingContext {
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

fn make_vbox<C, F>(name: &str, bounds: Bounds) -> View<C, F> {
    View {
        name: name.to_string(),
        title: name.to_string(),
        bounds,
        visible: true,
        draw: None,
        draw2: Some(|e|{
            e.ctx.fill_rect(&e.view.bounds, &e.theme.panel_bg);
        }),
        input: None,
        state: None,
        layout: Some(|evt|{
                if let Some(parent) = evt.scene.get_view_mut(evt.target) {
                    let mut y = 0;
                    let bounds = parent.bounds;
                    let kids = evt.scene.get_children(evt.target);
                    for kid in kids {
                        if let Some(ch) = evt.scene.get_view_mut(&kid) {
                            ch.bounds.x = 0;
                            ch.bounds.y = y;
                            ch.bounds.w = bounds.w;
                            y += ch.bounds.h;
                        }
                    }
                }
        }),
    }
}

struct MenuState {
    data:Vec<String>,
    selected:usize,
}
const vh:i32 = 30;
fn make_menuview<C, F>(name:&str, data:Vec<String>) -> View<C, F> {
    View {
        name: name.into(),
        title: name.into(),
        bounds: Bounds {
            x:0,
            y:0,
            w:100,
            h:(data.len() as i32) * vh,
        },
        visible:true,
        draw: Some(|view, ctx, theme| {
            ctx.fill_rect(&view.bounds, &theme.bg);
            if let Some(state) = &view.state {
                if let Some(state) = state.downcast_ref::<MenuState>() {
                    info!("menu state is {:?} {}",state.data, state.selected);
                    for (i,item) in (&state.data).iter().enumerate() {
                        let b = Bounds {
                            x: view.bounds.x+1,
                            y: view.bounds.y + (i as i32) * vh + 1,
                            w: view.bounds.w -2,
                            h: vh,
                        };
                        if state.selected == i {
                            ctx.fill_rect(&b, &theme.fg);
                            ctx.fill_text(&b, item.as_str(), &TextStyle::new(&theme.font, &theme.bg).with_halign(HAlign::Center));
                        }else {
                            ctx.fill_rect(&b, &theme.bg);
                            ctx.fill_text(&b, item.as_str(), &TextStyle::new(&theme.font, &theme.fg).with_halign(HAlign::Center));
                        }
                    }
                }
            }
            ctx.stroke_rect(&view.bounds, &theme.fg);
        }),
        draw2: None,
        input: Some(|event|{
            // info!("menu clicked at");
            match &event.event_type {
                EventType::Tap(pt) => {
                    // info!("tapped at {:?}",pt);
                    event.scene.mark_dirty_view(event.target);
                    if let Some(view) = event.scene.get_view_mut(event.target) {
                        // info!("the view is {} at {:?}",view.name, view.bounds);
                        let name = view.name.clone();
                        if view.bounds.contains(pt) {
                            // info!("I was clicked on. index is {}", pt.y/20);
                            let selected = (pt.y - view.bounds.y)/vh;
                            if let Some(state) = &mut view.state {
                                if let Some(state) = state.downcast_mut::<MenuState>() {
                                    if selected >= 0 && selected < state.data.len() as i32 {
                                        state.selected = selected as usize;
                                        info!("menu state is {:?}",state.selected);
                                        event.scene.set_focused(&name);
                                        return Some(Action::Command("selected".into()))
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    info!("unknown event type");
                }
            }
            None
        }),
        layout: Some(|event|{
            if let Some(parent) = event.scene.get_view_mut(event.target) {
                if let Some(state) = &parent.state {
                    if let Some(state) = state.downcast_ref::<MenuState>() {
                        parent.bounds.h = vh * (state.data.len() as i32);
                    }
                }
            };
        }),
        state: Some(Box::new(MenuState{data,selected:0})),
    }

}