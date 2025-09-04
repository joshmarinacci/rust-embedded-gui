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
use log::info;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::geometry::{Point, Size};
use esp_hal::i2c::master::{BusTimeout, I2c, Config as I2CConfig};
use mipidsi::interface::SpiInterface;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::{models::ST7789, Builder, Display, NoResetPin};
use static_cell::StaticCell;
use gui2::{connect_parent_child, draw_button_view, draw_panel_view, find_children, layout_vbox, pick_at, DrawingContext, EventType, GuiEvent, Scene, Theme, View};
use gui2::geom::{Bounds, Point as GPoint};
use gt911::Gt911Blocking;

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
    let mut scene: Scene<Rgb565> = make_gui_scene();


    let theme:Theme<Rgb565> = Theme {
        bg: Rgb565::WHITE,
        fg: Rgb565::BLACK,
        panel_bg: Rgb565::CSS_LIGHT_GRAY,
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
                    let mut evt:GuiEvent<Rgb565> = GuiEvent {
                        scene: &mut scene,
                        target,
                        event_type: EventType::Tap(pt)
                    };
                    info!("created event on target {:?} at {:?}",evt.target, evt.event_type);
                    if let Some(view) = evt.scene.get_view("target") {
                        if let Some(input) = view.input {
                            input(&mut evt);
                        }
                    }
                }
            }
        }

        let delay_start = Instant::now();
        if scene.dirty {
            ctx.display.clear(theme.panel_bg);
            draw_scene(&mut scene, &mut ctx, &theme);
        }
        while delay_start.elapsed() < Duration::from_millis(100) {}
    }
}

fn draw_scene(scene: &mut Scene<Rgb565>, ctx: &mut EmbeddedDrawingContext, theme: &Theme<Rgb565>) {
    let name = scene.rootId.clone();
    draw_view(scene, ctx, theme, &name);
    scene.dirty = false;
}
fn draw_view(scene: &mut Scene<Rgb565>, ctx: &mut EmbeddedDrawingContext, theme: &Theme<Rgb565>, name:&str) {
    if let Some(view) = scene.get_view(name) {
        // info!("drawing {} {:?}", name, view.bounds);
        (view.draw.unwrap())(view, ctx, &theme);
        let kids = find_children(&scene, &view.name);
        for kid in kids {
            draw_view(scene,ctx,theme,&kid);
        }
    }
}

fn make_gui_scene() -> Scene<Rgb565> {
    let mut scene: Scene<Rgb565> = Scene::new();
    let rootname = scene.rootId.clone();

    let mut panel = make_panel(Bounds{x:20,y:20,w:200,h:200});
    panel.name = "panel".into();


    let mut label = make_label("A Label");
    label.bounds.x = 10;
    label.bounds.y = 30;
    label.bounds.w = 100;
    label.bounds.h = 20;
    label.name = "label1".into();

    let mut button = make_button("A button");
    button.bounds.x = 10;
    button.bounds.y = 60;
    button.bounds.w = 100;
    button.bounds.h = 20;
    button.name = "button1".into();


    let mut textinput = make_text_input("type text here");
    textinput.bounds.x = 10;
    textinput.bounds.y = 90;
    textinput.bounds.w = 200;
    textinput.bounds.h = 30;
    textinput.name = "textinput".into();

    let mut menuview = make_menuview(vec!["first".into(),"second".into(),"third".into()]);
    menuview.bounds.x = 100;
    menuview.bounds.y = 30;
    menuview.name = "menuview".into();

    connect_parent_child(&mut scene,&rootname,&panel.name);
    connect_parent_child(&mut scene,&rootname,&label.name);
    connect_parent_child(&mut scene,&rootname,&button.name);
    connect_parent_child(&mut scene,&rootname,&textinput.name);
    connect_parent_child(&mut scene,&rootname,&menuview.name);

    scene.add_view(panel);
    scene.add_view(label);
    scene.add_view(button);
    scene.add_view(textinput);
    scene.add_view(menuview);

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
        let size = Size::new(bounds.w as u32, bounds.h as u32);
        Rectangle::new(pt,size)
            .into_styled(PrimitiveStyle::with_fill(*color))
            .draw(&mut self.display).unwrap();

    }

    fn strokeRect(&mut self, bounds: &Bounds, color: &Rgb565) {
        let pt = Point::new(bounds.x,bounds.y);
        let size = Size::new(bounds.w as u32, bounds.h as u32);
        Rectangle::new(pt,size)
            .into_styled(PrimitiveStyle::with_stroke(*color,1))
            .draw(&mut self.display).unwrap();
    }

    fn fillText(&mut self, bounds: &Bounds, text: &str, color: &Rgb565) {
        let style = MonoTextStyle::new(&FONT_6X10, *color);
        let mut pt = Point::new(bounds.x, bounds.y);
        pt.y += bounds.h / 2;
        pt.y += (FONT_6X10.baseline as i32)/2;
        let w = (FONT_6X10.character_size.width as i32) * (text.len() as i32);
        pt.x += (bounds.w - w) / 2;
        Text::new(text, pt, style)
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
        draw: Some(|view, ctx, theme|{
            ctx.fillRect(&view.bounds, &theme.bg);
            ctx.strokeRect(&view.bounds, &theme.fg);
            ctx.fillText(&view.bounds, &view.title, &theme.fg);
        }),
        input: Some(|event| {
            info!("button got input {:?}",event.target);
        }),
        state: None,
        layout: None,
    }
}

fn make_panel<C>(bounds:Bounds) -> View<C> {
    View {
        name:"something".into(),
        title: "some panel".into(),
        bounds: bounds,
        visible: true,
        draw: Some(|view, ctx, theme| {
            ctx.fillRect(&view.bounds, &theme.panel_bg);
            ctx.strokeRect(&view.bounds, &theme.fg);
        }),
        input: None,
        state: None,
        layout: None,
    }
}

fn make_label<C>(text:&str) -> View<C> {
    View {
        name:text.into(),
        title: text.into(),
        bounds: Bounds { x:0, y:0, w:10, h:20},
        visible:true,
        draw: Some(|view, ctx, theme| {
            ctx.fillText(&view.bounds, &view.title, &theme.fg);
        }),
        input: None,
        state: None,
        layout: None,
    }
}

fn make_text_input<C>(text:&str) -> View<C> {
    View {
        name: "text".into(),
        title: text.into(),
        bounds:Bounds {
            x: 0,
            y: 0,
            w: 200,
            h: 30,
        },
        visible: true,
        draw: Some(|view, ctx, theme| {
            ctx.fillRect(&view.bounds, &theme.bg);
            ctx.strokeRect(&view.bounds, &theme.fg);
            ctx.fillText(&view.bounds, &view.title,&theme.fg);
            // if view.focused {
            //     let cursor = Bounds {
            //         x: view.bounds.x + 20,
            //         y: view.bounds.y + 2,
            //         w: 2,
            //         h: view.bounds.h - 4,
            //     };
            //     ctx.fillRect(&cursor, &theme.fg);
            // }
        }),
        input: None,
        state: None,
        layout: None,
    }
}


struct MenuState {
    data:Vec<String>,
    selected:usize,
}
fn make_menuview<C>(data:Vec<String>) -> View<C> {
    View {
        name: "somemenu".into(),
        title: "somemenu".into(),
        bounds: Bounds {
            x:0,
            y:0,
            w:100,
            h:200,
        },
        visible:true,
        draw: Some(|view, ctx, theme| {
            ctx.fillRect(&view.bounds, &theme.bg);
            ctx.strokeRect(&view.bounds, &theme.fg);
            if let Some(state) = &view.state {
                if let Some(state) = state.downcast_ref::<MenuState>() {
                    info!("menu state is {:?}",state.data);
                    for (i,item) in (&state.data).iter().enumerate() {
                        let b = Bounds {
                            x: view.bounds.x,
                            y: view.bounds.y + (i as i32) * 30,
                            w: view.bounds.w,
                            h: 30,
                        };
                        if state.selected == i {
                            ctx.fillRect(&b,&theme.fg);
                            ctx.fillText(&b,item.as_str(),&theme.bg);
                        }else {
                            ctx.fillText(&b, item.as_str(), &theme.fg);
                        }
                    }
                }
            }
        }),
        input: Some(|v|{
            info!("menu clicked at");
            match &v.event_type {
                EventType::Tap(pt) => {
                    info!("tapped at {:?}",pt);
                    if let Some(view) = v.scene.get_view_mut(v.target) {
                        info!("the view is {} at {:?}",view.name, view.bounds);
                        if view.bounds.contains(pt) {
                            info!("I was clicked on. index is {}", pt.y/30);
                            let selected = pt.y/30;
                            if let Some(state) = &mut view.state {
                                if let Some(state) = state.downcast_mut::<MenuState>() {
                                    info!("menu state is {:?}",state.data);
                                    if selected >= 0 && selected < state.data.len() as i32 {
                                        state.selected = selected as usize;
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
        }),
        layout: None,
        state: Some(Box::new(MenuState{data,selected:0})),
    }

}