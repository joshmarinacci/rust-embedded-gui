#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

extern crate alloc;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::convert::Infallible;
use core::ops::Add;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::Level::{High, Low};
use esp_hal::gpio::{Input, InputConfig, Output, OutputConfig, Pull};
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::{Duration, Instant, Rate};
use esp_hal::{Blocking, main};
use iris_ui::Action;
use iris_ui::EventType;
use iris_ui::GuiEvent;
use iris_ui::Theme;
use iris_ui::button::make_button;
use iris_ui::geom::{Bounds, Insets, Point as GPoint, Size as GSize};
use iris_ui::gfx::DrawingContext;
use iris_ui::gfx::TextStyle;
use iris_ui::label::make_label;
use iris_ui::scene::Scene;
use iris_ui::scene::draw_scene;
use iris_ui::scene::pick_at;
use iris_ui::text_input::make_text_input;
use iris_ui::view::{Align, Flex, View, ViewId};
use log::info;

use embedded_graphics::geometry::Point as EPoint;
use embedded_graphics::mock_display::MockDisplay;
use embedded_graphics::mono_font::ascii::{FONT_7X13_BOLD, FONT_9X15};
use embedded_graphics::mono_font::{MonoFont, MonoTextStyleBuilder};
use embedded_graphics::primitives::{Line, PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Alignment, Baseline, TextStyleBuilder};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use esp_hal::dma::DmaAlignmentError::Size;
use esp_hal::i2c::master::{BusTimeout, Config as I2CConfig, I2c};
use mipidsi::interface::SpiInterface;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::{Builder, Display, NoResetPin, models::ST7789};
use static_cell::StaticCell;

use gt911::Gt911Blocking;
use iris_ui::device::EmbeddedDrawingContext;
use iris_ui::layouts::layout_hbox;
use iris_ui::panel::draw_std_panel;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

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

    let mut TFT_CS = Output::new(peripherals.GPIO12, High, OutputConfig::default());
    TFT_CS.set_high();
    let tft_dc = Output::new(peripherals.GPIO11, Low, OutputConfig::default());
    let mut tft_enable = Output::new(peripherals.GPIO42, High, OutputConfig::default());
    tft_enable.set_high();

    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default().with_frequency(Rate::from_mhz(40)),
    )
    .unwrap()
    .with_sck(peripherals.GPIO40)
    .with_miso(Input::new(
        peripherals.GPIO38,
        InputConfig::default().with_pull(Pull::Up),
    ))
    .with_mosi(peripherals.GPIO41);

    static DISPLAY_BUF: StaticCell<[u8; 512]> = StaticCell::new();
    let buffer = DISPLAY_BUF.init([0u8; 512]);

    info!("setting up the display");
    let spi_delay = Delay::new();
    let spi_device = ExclusiveDevice::new(spi, TFT_CS, spi_delay).unwrap();
    let di = SpiInterface::new(spi_device, tft_dc, buffer);
    info!("building");
    let mut display = Builder::new(ST7789, di)
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

    let mut ctx = EmbeddedDrawingContext::new(&mut display);
    let mut scene = make_gui_scene();

    let theme = Theme {
        bg: Rgb565::WHITE,
        fg: Rgb565::BLACK,
        selected_bg: Rgb565::WHITE,
        selected_fg: Rgb565::BLACK,
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
                let pt = GPoint::new(320 - point.y as i32, 240 - point.x as i32);
                let targets = pick_at(&mut scene, &pt);
                info!("clicked on targets {:?}", targets);
                if let Some((target, pt)) = targets.last() {
                    let mut evt = GuiEvent {
                        scene: &mut scene,
                        target,
                        event_type: EventType::Tap(pt.clone()),
                        action: None,
                    };
                    info!(
                        "created event on target {:?} at {:?}",
                        evt.target, evt.event_type
                    );
                    if let Some(view) = evt.scene.get_view(evt.target) {
                        if let Some(input) = view.input {
                            input(&mut evt);
                        }
                    }
                }
            }
        }

        let delay_start = Instant::now();
        ctx.clip = scene.dirty_rect.clone();
        draw_scene(&mut scene, &mut ctx, &theme);
        while delay_start.elapsed() < Duration::from_millis(100) {}
    }
}

fn make_gui_scene() -> Scene {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

    let panel = View {
        name: ViewId::new("panel"),
        bounds: Bounds::new(20, 20, 200, 200),
        draw: Some(draw_std_panel),
        padding: Insets::new_same(5),
        h_flex: Flex::Resize,
        v_flex: Flex::Resize,
        layout: Some(layout_hbox),
        ..Default::default()
    };

    scene.add_view_to_parent(
        make_label("label1", "A Label").position_at(10, 30),
        &panel.name,
    );

    scene.add_view_to_root(make_button(&"button1".into(), "A button").position_at(10, 60));
    scene.add_view_to_root(make_button(&"button2".into(), "A button").position_at(10, 120));

    scene.add_view_to_root(make_button(&"button3".into(), "A button").position_at(10, 200));
    scene.mark_dirty_all();

    scene.add_view_to_parent(
        make_text_input("textinput", "type text here").position_at(10, 90),
        &panel.name,
    );

    scene.add_view_to_parent(
        make_menuview(
            "menuview",
            vec!["first".into(), "second".into(), "third".into()],
        )
        .position_at(100, 30),
        &panel.name,
    );
    scene.add_view_to_root(panel);

    scene
}

struct MenuState {
    data: Vec<String>,
    selected: usize,
}
const vh: i32 = 30;
fn make_menuview(name: &'static str, data: Vec<String>) -> View {
    View {
        name: ViewId::new(name),
        title: name.into(),
        bounds: Bounds::new(0, 0, 100, data.len() as i32 * vh),
        visible: true,
        draw: Some(|e| {
            e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
            if let Some(state) = &e.view.state {
                if let Some(state) = state.downcast_ref::<MenuState>() {
                    info!("menu state is {:?} {}", state.data, state.selected);
                    for (i, item) in (&state.data).iter().enumerate() {
                        let b = Bounds {
                            position: GPoint {
                                x: e.view.bounds.position.x + 1,
                                y: e.view.bounds.position.y + (i as i32) * vh + 1,
                            },
                            size: GSize {
                                w: e.view.bounds.size.w - 2,
                                h: vh,
                            },
                        };
                        if state.selected == i {
                            e.ctx.fill_rect(&b, &e.theme.fg);
                            e.ctx.fill_text(
                                &b,
                                item.as_str(),
                                &TextStyle::new(&e.theme.font, &e.theme.bg)
                                    .with_halign(Align::Center),
                            );
                        } else {
                            e.ctx.fill_rect(&b, &e.theme.bg);
                            e.ctx.fill_text(
                                &b,
                                item.as_str(),
                                &TextStyle::new(&e.theme.font, &e.theme.fg)
                                    .with_halign(Align::Center),
                            );
                        }
                    }
                }
            }
            e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
        }),
        input: Some(|event| {
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
                            let selected = (pt.y - view.bounds.position.y) / vh;
                            if let Some(state) = &mut view.state {
                                if let Some(state) = state.downcast_mut::<MenuState>() {
                                    if selected >= 0 && selected < state.data.len() as i32 {
                                        state.selected = selected as usize;
                                        info!("menu state is {:?}", state.selected);
                                        event.scene.set_focused(&name);
                                        return Some(Action::Command("selected".into()));
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
        layout: Some(|event| {
            if let Some(parent) = event.scene.get_view_mut(event.target) {
                if let Some(state) = &parent.state {
                    if let Some(state) = state.downcast_ref::<MenuState>() {
                        parent.bounds.size.h = vh * (state.data.len() as i32);
                    }
                }
            };
        }),
        state: Some(Box::new(MenuState { data, selected: 0 })),
        ..Default::default()
    }
}
