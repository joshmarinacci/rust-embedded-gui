use embedded_graphics::Drawable;
use embedded_graphics::geometry::{Point as EPoint, Size};
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::mono_font::ascii::{FONT_5X7, FONT_6X10, FONT_7X13_BOLD, FONT_9X15, FONT_9X15_BOLD};
use embedded_graphics::mono_font::iso_8859_9::FONT_7X13;
use embedded_graphics::pixelcolor::{Rgb565, Rgb888};
use embedded_graphics::prelude::Primitive;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::prelude::WebColors;
use embedded_graphics::primitives::{Line, PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Alignment, Text, TextStyleBuilder, TextStyle as ETextStyle, Baseline};
use rust_embedded_gui::button::make_button;
use rust_embedded_gui::geom::{Bounds, Point as GPoint};
use rust_embedded_gui::scene::{
    click_at, draw_scene, event_at_focused, layout_scene, EventResult, Scene,
};
use rust_embedded_gui::toggle_button::make_toggle_button;
use rust_embedded_gui::toggle_group::{make_toggle_group, SelectOneOfState};
use rust_embedded_gui::{Action, EventType, Theme};
use std::ops::Add;

#[cfg(feature = "std")]
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use env_logger::fmt::style::Color::Rgb;
use env_logger::Target;
use log::{info, LevelFilter};
use rust_embedded_gui::gfx::{DrawingContext, HAlign, TextStyle, VAlign};
use rust_embedded_gui::grid::{make_grid_panel, GridLayoutState, LayoutConstraint};
use rust_embedded_gui::label::make_label;
use rust_embedded_gui::list_view::make_list_view;
use rust_embedded_gui::panel::{layout_hbox, layout_vbox, make_panel, PanelState};
use rust_embedded_gui::text_input::make_text_input;
use rust_embedded_gui::view::View;

const SMALL_FONT_BUTTON: &str = "small_font";
const MEDIUM_FONT_BUTTON: &str = "medium_font";
const LARGE_FONT_BUTTON: &str = "large_font";

const TABBED_PANEL: &str = "tabbed-panel";
const BUTTONS_PANEL: &str = "buttons";
const LAYOUT_PANEL: &str = "layout-panel";
const LISTS_PANEL: &str = "lists-panel";
const INPUTS_PANEL: &str = "input-panel";
const THEMES_PANEL: &str = "themes-panel";

const POPUP_BUTTON: &str = "list-button";
const POPUP_MENU:&str = "popup-menu";
fn make_scene() -> Scene {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

    let mut tabbed_panel = make_tabs(
        TABBED_PANEL,
        vec!["buttons", "layouts", "lists", "inputs", "themes"],
        Bounds {
            x: 10,
            y: 10,
            w: 320 - 20,
            h: 180,
        },
    );

    let tabs = make_toggle_group(
        "tabs",
        vec!["buttons", "layouts", "lists", "inputs", "themes"],
        0,
    );
    scene.add_view_to_parent(tabs, &tabbed_panel.name);

    {
        let mut grid = make_grid_panel(BUTTONS_PANEL);
        grid.bounds = Bounds::new(50, 50, 100, 100);
        let mut grid_layout = GridLayoutState::new_row_column(3, 30, 2, 100);
        grid_layout.debug = false;
        grid_layout.padding = 10;
        grid_layout.border = false;

        let label1 = make_label("label1", "A Label");
        grid_layout.place_at_row_column(&label1.name, 0, 0);
        scene.add_view_to_parent(label1, &grid.name);

        let button1 = make_button("button1", "Basic Button");
        grid_layout.place_at_row_column(&button1.name, 1, 0);
        scene.add_view_to_parent(button1, &grid.name);

        let button2 = make_toggle_button("toggle1", "Toggle");
        grid_layout.place_at_row_column(&button2.name, 1, 1);
        scene.add_view_to_parent(button2, &grid.name);

        let button3 = make_toggle_group("toggle2", vec!["Apple", "Ball", "Car"], 1);
        grid_layout.constraints.insert((&button3.name).into(),LayoutConstraint {
            row:2,
            col: 0,
            col_span: 2,
            row_span: 1,
            v_align: VAlign::Center,
            h_align: HAlign::Center,
        });
        scene.add_view_to_parent(button3, &grid.name);

        grid.state = Some(Box::new(grid_layout));
        scene.add_view_to_parent(grid, &tabbed_panel.name);
    }
    {
        let mut wrapper = make_panel(LAYOUT_PANEL, Bounds::new(0, 50, 100, 100));
        wrapper.state = Some(Box::new(PanelState{
            padding: 5,
            debug: false,
            border:false,
            bg: true,
            gap: 5,
            halign: HAlign::Center,
            valign: VAlign::Top,
        }));
        wrapper.layout = Some(layout_hbox);

        let mut col1 = make_column("vbox2");
        scene.add_view_to_parent(make_label("vbox-label", "vbox layout"), &col1.name);
        let mut vbox = make_panel("vbox", Bounds::new(0,0,100,100));
        vbox.layout = Some(layout_vbox);
        scene.add_view_to_parent(make_button("vbox-button1", "A"), &vbox.name);
        scene.add_view_to_parent(make_button("vbox-button2", "B"), &vbox.name);
        scene.add_view_to_parent(make_button("vbox-button3", "C"), &vbox.name);
        scene.add_view_to_parent(vbox, &col1.name);
        scene.add_view_to_parent(col1, &wrapper.name);

        let mut col2 = make_column("vbox3");
        scene.add_view_to_parent(make_label("hbox-label", "hbox layout"), &col2.name);

        let mut hbox = make_panel("hbox", Bounds::new(0,0,100,100));
        hbox.layout = Some(layout_hbox);
        scene.add_view_to_parent(make_button("hbox-button1", "A"), &hbox.name);
        scene.add_view_to_parent(make_button("hbox-button2", "B"), &hbox.name);
        scene.add_view_to_parent(make_button("hbox-button3", "C"), &hbox.name);
        scene.add_view_to_parent(hbox, &col2.name);
        scene.add_view_to_parent(col2, &wrapper.name);

        scene.add_view_to_parent(wrapper,&tabbed_panel.name);
    }
    {
        let mut wrapper = make_panel(LISTS_PANEL, Bounds::new(0, 50, 100, 100));
        wrapper.state = Some(Box::new(PanelState{
            padding: 5,
            debug: false,
            border:false,
            bg: true,
            gap: 5,
            halign: HAlign::Left,
            valign: VAlign::Top,
        }));
        wrapper.layout = Some(layout_hbox);
        let col1 = make_column("lists-col1");
        scene.add_view_to_parent(make_label("lists-label", "Lists"), &col1.name);
        let button = make_button(POPUP_BUTTON,"Open Popup");
        scene.add_view_to_parent(button,&col1.name);
        scene.add_view_to_parent(col1,&wrapper.name);

        let list = make_list_view("list-view",vec!["First","Second","Third","Fourth","Fifth"],1);
        scene.add_view_to_parent(list,&wrapper.name);
        scene.add_view_to_parent(wrapper, &tabbed_panel.name);
    }
    {
        let mut panel = make_panel(INPUTS_PANEL, Bounds::new(0, 50, 100, 100));
        if let Some(state) = panel.get_state::<PanelState>() {
            state.border = false;
            state.gap = 5;
            state.bg = false;
        }

        scene.add_view_to_parent(
            make_text_input("textinput", "input").position_at(10, 10),
            &panel.name,
        );
        scene.add_view_to_parent(panel, &tabbed_panel.name);
    }
    {
        let mut panel = make_panel(THEMES_PANEL, Bounds::new(0, 50, 100, 100));
        panel.layout = Some(layout_vbox);
        panel.state = Some(Box::new(PanelState{
            padding: 10,
            debug: false,
            border:false,
            bg:true,
            gap: 10,
            halign: HAlign::Center,
            valign: VAlign::Bottom,
        }));

        scene.add_view_to_parent(
            make_label("themes-label", "Themes").position_at(30, 90),
            &panel.name,
        );
        scene.add_view_to_parent(make_button("light-theme", "Light"), &panel.name);
        scene.add_view_to_parent(make_button("dark-theme", "Dark"), &panel.name);
        scene.add_view_to_parent(make_button("colorful-theme", "Colorful"), &panel.name);
        scene.add_view_to_parent(panel, &tabbed_panel.name);
    }

    scene.add_view_to_root(tabbed_panel);

    let mut font_buttons = make_panel("font_buttons", Bounds::new(30, 200, 200, 30));
    font_buttons.layout = Some(layout_hbox);
    if let Some(state) = font_buttons.get_state::<PanelState>() {
        state.border = false;
        state.gap = 5;
        state.bg = false;
    }
    scene.add_view_to_parent(make_button(SMALL_FONT_BUTTON, "Small"), &font_buttons.name);
    scene.add_view_to_parent(make_button(MEDIUM_FONT_BUTTON, "Medium"), &font_buttons.name);
    scene.add_view_to_parent(make_button(LARGE_FONT_BUTTON, "Large"), &font_buttons.name);
    scene.add_view_to_root(font_buttons);

    if let Some(state) = scene.get_view_state::<SelectOneOfState>(TABBED_PANEL) {
        state.selected = 2;
    }

    scene
}

fn make_column(name: &str) -> View {
    let mut panel = make_panel(name, Bounds::new(0, 0, 100, 100));
    if let Some(state) = panel.get_state::<PanelState>() {
        state.border = false;
        state.gap = 5;
        state.bg = false;
    }
    panel.layout = Some(layout_vbox);
    panel
}

fn make_tabs(name: &str, tabs: Vec<&str>, bounds: Bounds) -> View {
    View {
        name: name.into(),
        title: name.into(),
        bounds,
        visible: true,
        state: Some(SelectOneOfState::new_with(tabs, 0)),
        input: None,
        layout: Some(|e| {
            if let Some(state) = e.scene.get_view_state::<SelectOneOfState>(e.target) {
                let selected = state.selected;
                if let Some(parent) = e.scene.get_view_mut(e.target) {
                    let bounds = parent.bounds;
                    let mut tabs_height = 50;
                    for (i, kid) in e.scene.get_children(e.target).iter().enumerate() {
                        if let Some(ch) = e.scene.get_view_mut(&kid) {
                            if kid == "tabs" {
                                ch.bounds = Bounds::new(0, 0, bounds.w, ch.bounds.h);
                                tabs_height = ch.bounds.h;
                                ch.visible = true;
                            } else {
                                ch.bounds = Bounds::new(
                                    0+1,
                                    0 + tabs_height+1,
                                    bounds.w-2,
                                    bounds.h - tabs_height -2,
                                );
                                ch.visible = false;
                                if i == selected + 1 {
                                    ch.visible = true;
                                }
                            }
                        }
                    }
                }
            }
        }),
        draw: Some(|e| {
            e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
            e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
        }),
    }
}

struct SimulatorDrawingContext<'a> {
    pub clip: Bounds,
    display: &'a mut SimulatorDisplay<Rgb565>,
    offset: EPoint,
}

impl SimulatorDrawingContext<'_> {
    fn new(display: &mut SimulatorDisplay<Rgb565>) -> SimulatorDrawingContext {
        SimulatorDrawingContext {
            display,
            clip: Bounds::new_empty(),
            offset: EPoint::new(0, 0),
        }
    }
}

fn bounds_to_rect(bounds: &Bounds) -> Rectangle {
    Rectangle::new(
        EPoint::new(bounds.x, bounds.y),
        Size::new(bounds.w as u32, bounds.h as u32),
    )
}

impl DrawingContext for SimulatorDrawingContext<'_> {
    fn fill_rect(&mut self, bounds: &Bounds, color: &Rgb565) {
        let mut display = &mut self.display;
        let mut display = display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);
        bounds_to_rect(bounds)
            .into_styled(PrimitiveStyle::with_fill(*color))
            .draw(&mut display)
            .unwrap();
    }
    fn stroke_rect(&mut self, bounds: &Bounds, color: &Rgb565) {
        let mut display = &mut self.display;
        let mut display = display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);
        bounds_to_rect(bounds)
            .into_styled(PrimitiveStyle::with_stroke(*color, 1))
            .draw(&mut display)
            .unwrap();
    }

    fn line(&mut self, start: &GPoint, end: &GPoint, color: &Rgb565) {
        let mut display = &mut self.display;
        let mut display = display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);
        let line = Line::new(EPoint::new(start.x,start.y),EPoint::new(end.x,end.y));
        line.into_styled(PrimitiveStyle::with_stroke(*color,1)).draw(&mut display).unwrap();
    }

    fn fill_text(&mut self, bounds: &Bounds, text: &str, text_style: &TextStyle) {
        let mut display = &mut self.display;
        let mut display = display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);

        let mut text_builder = MonoTextStyleBuilder::new()
            .font(text_style.font)
            .text_color(*text_style.color);
        if text_style.underline {
            text_builder = text_builder.underline();
        }
        let style = text_builder.build(); // MonoTextStyle::new(&FONT_6X10,  *text_style.color);
        let mut pt = EPoint::new(bounds.x, bounds.y);
        pt.y += bounds.h / 2;
        pt.y += (FONT_6X10.baseline as i32) / 2;

        let w = (FONT_6X10.character_size.width as i32) * (text.len() as i32);

        match text_style.halign {
            HAlign::Left => {
                pt.x += 5;
            }
            HAlign::Center => {
                pt.x += (bounds.w - w) / 2;
            }
            HAlign::Right => {}
        }

        Text::new(text, pt, style).draw(&mut display).unwrap();
    }

    fn text(&mut self, text: &str, position: &GPoint, style: &TextStyle) {
        let mut display = &mut self.display;
        let mut display = display.clipped(&bounds_to_rect(&self.clip));
        let mut display = display.translated(self.offset);
        let mut pt = EPoint::new(position.x,position.y);
        let mut text_builder = MonoTextStyleBuilder::new()
            .font(style.font)
            .text_color(*style.color);
        if style.underline {
            text_builder = text_builder.underline();
        }
        let estyle = text_builder.build();
        let etext = Text {
            position:pt,
            text: text,
            character_style: estyle,
            text_style: TextStyleBuilder::new().alignment(Alignment::Center).baseline(Baseline::Middle).build(),
        };
        etext.draw(&mut display).unwrap();
    }

    fn translate(&mut self, offset: &GPoint) {
        self.offset = self.offset.add(EPoint::new(offset.x, offset.y));
    }
}

fn main() -> Result<(), std::convert::Infallible> {
    env_logger::Builder::new()
        .target(Target::Stdout) // <-- redirects to stdout
        .filter(None, LevelFilter::Info)
        .init();

    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));

    let mut scene = make_scene();
    let mut theme = Theme {
        bg: Rgb565::WHITE,
        fg: Rgb565::BLACK,
        selected_bg: Rgb565::BLUE,
        selected_fg: Rgb565::WHITE,
        panel_bg: Rgb565::CSS_LIGHT_GRAY,
        font: FONT_7X13,
        bold_font: FONT_7X13_BOLD,
    };

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Simulator Test", &output_settings);
    'running: loop {
        let mut ctx: SimulatorDrawingContext = SimulatorDrawingContext::new(&mut display);
        ctx.clip = scene.dirty_rect.clone();
        layout_scene(&mut scene, &theme);
        draw_scene(&mut scene, &mut ctx, &theme);
        window.update(&display);
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown {
                    keycode, keymod, ..
                } => {
                    let key: u8 = keydown_to_char(keycode, keymod);
                    println!(
                        "keyboard event {} {} {:?}",
                        keycode.name(),
                        key,
                        String::from(key as char)
                    );
                    if key > 0 {
                        if let Some(result) = event_at_focused(&mut scene, EventType::Keyboard(key))
                        {
                            println!("got input from {:?}", result);
                        }
                    }
                }
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

fn keydown_to_char(keycode: Keycode, keymod: Mod) -> u8 {
    println!("keycode as number {}", keycode.into_i32());
    let ch = keycode.into_i32();
    if ch <= 0 {
        return 0;
    }
    let shifted = keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD);

    if let Some(ch) = char::from_u32(ch as u32) {
        if ch.is_alphabetic() {
            return if shifted {
                ch.to_ascii_uppercase() as u8
            } else {
                ch.to_ascii_lowercase() as u8
            };
        }
        if ch.is_ascii_graphic() {
            return ch as u8;
        }
    }
    match keycode {
        Keycode::Backspace => 8,
        Keycode::SPACE => b' ',
        _ => {
            println!("not supported: {keycode}");
            0
        }
    }
}

fn handle_events(result: EventResult, scene: &mut Scene, theme: &mut Theme) {
    let (name, action) = result;
    println!("result of event {:?} from {name}", action);
    if name == SMALL_FONT_BUTTON {
        theme.font = FONT_5X7;
        theme.bold_font = FONT_5X7;
        scene.mark_layout_dirty();
    }
    if name == MEDIUM_FONT_BUTTON {
        theme.font = FONT_6X10;
        theme.bold_font = FONT_6X10;
        scene.mark_layout_dirty();
    }
    if name == LARGE_FONT_BUTTON {
        theme.font = FONT_7X13;
        theme.bold_font = FONT_7X13_BOLD;
        scene.mark_layout_dirty();
    }
    if name == "light-theme" {
        theme.bg = Rgb565::WHITE;
        theme.fg = Rgb565::BLACK;
        theme.panel_bg = Rgb565::CSS_LIGHT_GRAY;
        theme.selected_bg = Rgb565::CSS_CORNFLOWER_BLUE;
        theme.selected_fg = Rgb565::WHITE;
        scene.mark_dirty_all();
    }
    if name == "dark-theme" {
        theme.bg = Rgb565::from(Rgb888::new(50, 50, 50));
        theme.fg = Rgb565::WHITE;
        theme.panel_bg = Rgb565::BLACK;
        theme.selected_bg = Rgb565::CSS_DARK_BLUE;
        theme.selected_fg = Rgb565::WHITE;
        scene.mark_dirty_all();
    }
    if name == "colorful-theme" {
        theme.bg = Rgb565::CSS_MISTY_ROSE;
        theme.fg = Rgb565::CSS_DARK_BLUE;
        theme.panel_bg = Rgb565::CSS_ANTIQUE_WHITE;
        theme.selected_bg = Rgb565::CSS_DARK_MAGENTA;
        theme.selected_fg = Rgb565::CSS_LIGHT_YELLOW;
        scene.mark_dirty_all();
    }
    if name == POPUP_BUTTON {
        let menu = make_list_view(POPUP_MENU,vec!["Item 1", "Item 2", "Item 3"],0)
            .position_at(50,50);
        scene.add_view_to_root(menu);
    }
    if name == POPUP_MENU {
        scene.remove_view(POPUP_MENU);
    }

    if name == "tabs" {
        match action {
            Action::Command(cmd) => {
                for kid in scene.get_children(TABBED_PANEL) {
                    if kid != "tabs" {
                        scene.hide_view(&kid);
                    }
                }
                match cmd.as_str() {
                    "buttons" => scene.show_view(BUTTONS_PANEL),
                    "layouts" => scene.show_view(LAYOUT_PANEL),
                    "lists" => scene.show_view(LISTS_PANEL),
                    "inputs" => scene.show_view(INPUTS_PANEL),
                    "themes" => scene.show_view(THEMES_PANEL),
                    &_ => {
                        println!("tab not handled");
                    }
                }
            }
            Action::Generic => {}
        }
    }
}
