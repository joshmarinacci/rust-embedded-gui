#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use alloc::string::{String, ToString};
use embedded_hal_bus::spi::{ExclusiveDevice, RefCellDevice};
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::Level::{High, Low};
use esp_hal::gpio::{Input, InputConfig, Output, OutputConfig, Pull};
use esp_hal::{main, Blocking};
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::{Duration, Instant, Rate};
use log::info;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::geometry::{Point, Size};
use mipidsi::interface::SpiInterface;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::{models::ST7789, Builder, Display, NoResetPin};
use static_cell::StaticCell;
use gui2::{draw_button_view, draw_panel_view, find_children, layout_vbox, DrawingContext, Scene, Theme, View};
use gui2::geom::Bounds;

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
    let mut scene: Scene<Rgb565> = Scene::new();
    scene.add_view(make_vbox(
        "parent",
        Bounds {
            x: 10,
            y: 10,
            w: 100,
            h: 100,
        },
    ));
    // add button 1
    scene.add_view(make_button("button1"));
    // add button 2
    scene.add_view(make_button("button2"));


    let theme:Theme<Rgb565> = Theme {
        bg: Rgb565::WHITE,
        fg: Rgb565::BLACK,
        panel_bg: Rgb565::GREEN,
    };
    loop {
        info!("sleeping");
        let delay_start = Instant::now();
        ctx.display.clear(Rgb565::BLACK);

        if let Some(root) = scene.get_view(&scene.rootId) {
            (root.draw.unwrap())(root, &mut ctx, &theme);
            let kids = find_children(&scene, &root.name);
            for kid in kids {
                if let Some(kid) = scene.get_view(&kid) {
                    (kid.draw.unwrap())(root, &mut ctx, &theme);
                }
            }
            scene.dirty = false;
        }
        while delay_start.elapsed() < Duration::from_millis(500) {}
    }
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
        }
    }
}

impl DrawingContext<Rgb565> for EmbeddedDrawingContext {
    fn fillRect(&mut self, bounds: &Bounds, color: &Rgb565) {
        let pt = Point::new(bounds.x,bounds.y);
        let size = Size::new(bounds.w as u32, bounds.y as u32);
        Rectangle::new(pt,size)
            .into_styled(PrimitiveStyle::with_fill(*color))
            .draw(&mut self.display).unwrap();

    }

    fn strokeRect(&mut self, bounds: &Bounds, color: &Rgb565) {
        let pt = Point::new(bounds.x,bounds.y);
        let size = Size::new(bounds.w as u32, bounds.y as u32);
        Rectangle::new(pt,size)
            .into_styled(PrimitiveStyle::with_stroke(*color,1))
            .draw(&mut self.display).unwrap();
    }

    fn fillText(&mut self, bounds: &Bounds, text: &str, color: &Rgb565) {
        let style = MonoTextStyle::new(&FONT_6X10, *color);
        let pt = Point::new(bounds.x,bounds.y);
        let size = Size::new(bounds.w as u32, bounds.y as u32);
        Text::new(text, Point::new(20, 30), style)
            .draw(&mut self.display)
            .unwrap();
    }
}

fn make_vbox<C>(name: &str, bounds: Bounds) -> View<C> {
    View {
        name: name.to_string(),
        title: name.to_string(),
        bounds,
        visible: true,
        draw: Some(draw_panel_view),
        input: None,
        state: None,
        layout: Some(layout_vbox),
    }
}

fn make_button<C>(name: &str) -> View<C> {
    View {
        name: name.to_string(),
        title: name.to_string(),
        bounds: Bounds {
            x: 0,
            y: 0,
            w: 20,
            h: 20,
        },
        visible: true,
        draw: Some(draw_button_view),
        input: None,
        state: None,
        layout: None,
    }
}
