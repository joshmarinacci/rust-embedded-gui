#[cfg(feature = "std")]
use embedded_graphics::geometry::{Point as EPoint, Size};
use embedded_graphics::mono_font::ascii::{
    FONT_5X7, FONT_6X10, FONT_7X13_BOLD, FONT_9X15, FONT_9X15_BOLD,
};
use embedded_graphics::mono_font::iso_8859_9::FONT_7X13;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::{Rgb565, Rgb888};
use embedded_graphics::prelude::Primitive;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::prelude::WebColors;
use embedded_graphics::primitives::{Line, PrimitiveStyle, Rectangle};
use embedded_graphics::Drawable;
use rust_embedded_gui::button::make_button;
use rust_embedded_gui::geom::{Bounds, Insets, Point as GPoint};
use rust_embedded_gui::scene::{
    click_at, draw_scene, event_at_focused, layout_scene, EventResult, Scene,
};
use rust_embedded_gui::toggle_button::make_toggle_button;
use rust_embedded_gui::toggle_group::{layout_toggle_group, make_toggle_group, SelectOneOfState};
use rust_embedded_gui::{Action, EventType, KeyboardAction, Theme};
use std::convert::Into;

use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use env_logger::fmt::style::Color::Rgb;
use env_logger::Target;
use log::{info, LevelFilter};
use rust_embedded_gui::device::EmbeddedDrawingContext;
use rust_embedded_gui::grid::{make_grid_panel, GridLayoutState, LayoutConstraint};
use rust_embedded_gui::label::make_label;
use rust_embedded_gui::layouts::{layout_hbox, layout_std_panel, layout_tabbed_panel, layout_vbox};
use rust_embedded_gui::list_view::make_list_view;
use rust_embedded_gui::panel::draw_std_panel;
use rust_embedded_gui::text_input::make_text_input;
use rust_embedded_gui::view::Align::{Center, Start};
use rust_embedded_gui::view::Flex::{Intrinsic, Resize};
use rust_embedded_gui::view::{Align, Flex, View, ViewId};

const SMALL_FONT_BUTTON: &'static ViewId = &ViewId::new("small_font");
const MEDIUM_FONT_BUTTON: &'static ViewId = &ViewId::new("medium_font");
const LARGE_FONT_BUTTON: &'static ViewId = &ViewId::new("large_font");

const TABBED_PANEL: &'static ViewId = &ViewId::new("tabbed-panel");
const BUTTONS_PANEL: &'static ViewId = &ViewId::new("buttons");
const LAYOUT_PANEL: &'static ViewId = &ViewId::new("layout-panel");
const LISTS_PANEL: &'static ViewId = &ViewId::new("lists-panel");
const INPUTS_PANEL: &'static ViewId = &ViewId::new("input-panel");
const THEMES_PANEL: &'static ViewId = &ViewId::new("themes-panel");

const POPUP_BUTTON: &'static ViewId = &ViewId::new("list-button");
const POPUP_MENU: &'static ViewId = &ViewId::new("popup-menu");
fn make_scene() -> Scene {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

    let mut tabbed_panel: View = View {
        name: TABBED_PANEL.clone(),
        bounds: Bounds::new(10, 10, 320 - 20, 180),
        h_flex: Flex::Intrinsic,
        v_flex: Flex::Intrinsic,
        draw: Some(|e| {
            e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
            e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
        }),
        layout: Some(layout_tabbed_panel),
        ..Default::default()
    };

    let tabs_id = ViewId::new("tabs");
    let tabs = make_toggle_group(
        &tabs_id,
        vec!["buttons", "layouts", "lists", "inputs", "themes"],
        0,
    );
    scene.add_view_to_parent(tabs, &tabbed_panel.name);

    {
        let mut grid = make_grid_panel(BUTTONS_PANEL);
        grid.padding = Insets::new_same(10);
        grid.h_flex = Resize;
        grid.v_flex = Resize;
        let mut grid_layout = GridLayoutState::new_row_column(3, 30, 2, 100);
        grid_layout.debug = false;

        let label1 = make_label("label1", "A Label");
        grid_layout.place_at_row_column(&label1.name, 0, 0);
        scene.add_view_to_parent(label1, &grid.name);

        let button1 = make_button(&ViewId::new("button1"), "Basic Button");
        grid_layout.place_at_row_column(&button1.name, 1, 0);
        scene.add_view_to_parent(button1, &grid.name);

        let button2 = make_toggle_button(&ViewId::new("toggle1"), "Toggle");
        grid_layout.place_at_row_column(&button2.name, 1, 1);
        scene.add_view_to_parent(button2, &grid.name);

        let mut button3 =
            make_toggle_group(&ViewId::new("toggle2"), vec!["Apple", "Ball", "Car"], 1);
        button3.h_flex = Intrinsic;
        button3.h_align = Align::Center;
        grid_layout.constraints.insert(
            (&button3.name).clone(),
            LayoutConstraint {
                row: 2,
                col: 0,
                col_span: 2,
                row_span: 1,
            },
        );
        scene.add_view_to_parent(button3, &grid.name);

        grid.state = Some(Box::new(grid_layout));
        scene.add_view_to_parent(grid, &tabbed_panel.name);
    }
    {
        let mut wrapper = View {
            name: LAYOUT_PANEL.clone(),
            draw: Some(draw_std_panel),
            padding: Insets::new_same(5),
            h_flex: Flex::Resize,
            v_flex: Flex::Resize,
            layout: Some(layout_hbox),
            ..Default::default()
        };

        {
            let col1 = make_column("vbox2");
            scene.add_view_to_parent(make_label("vbox-label", "vbox layout"), &col1.name);
            let vbox = View {
                name: ViewId::new("vbox"),
                draw: Some(draw_std_panel),
                layout: Some(layout_vbox),
                ..Default::default()
            };
            scene.add_view_to_parent(make_button(&ViewId::new("vbox-button1"), "A"), &vbox.name);
            scene.add_view_to_parent(make_button(&ViewId::new("vbox-button2"), "B"), &vbox.name);
            scene.add_view_to_parent(make_button(&ViewId::new("vbox-button3"), "C"), &vbox.name);
            scene.add_view_to_parent(vbox, &col1.name);
            scene.add_view_to_parent(col1, &wrapper.name);
        }

        {
            let col2 = make_column("vbox3");
            scene.add_view_to_parent(make_label("hbox-label", "hbox layout"), &col2.name);
            let hbox = make_row("hbox");
            scene.add_view_to_parent(make_button(&ViewId::new("hbox-button1"), "A"), &hbox.name);
            scene.add_view_to_parent(make_button(&ViewId::new("hbox-button2"), "B"), &hbox.name);
            scene.add_view_to_parent(make_button(&ViewId::new("hbox-button3"), "C"), &hbox.name);
            scene.add_view_to_parent(hbox, &col2.name);
            scene.add_view_to_parent(col2, &wrapper.name);
        }

        wrapper.visible = false;
        scene.add_view_to_parent(wrapper, &tabbed_panel.name);
    }
    {
        let mut wrapper = View {
            name: LISTS_PANEL.clone(),
            layout: Some(layout_hbox),
            draw: Some(draw_std_panel),
            h_flex: Flex::Resize,
            v_flex: Flex::Resize,
            ..Default::default()
        };
        let col1 = make_column("lists-col1");
        scene.add_view_to_parent(make_label("lists-label", "Lists"), &col1.name);
        let button = make_button(POPUP_BUTTON, "Open Popup");
        scene.add_view_to_parent(button, &col1.name);
        scene.add_view_to_parent(col1, &wrapper.name);
        let list = make_list_view(
            &ViewId::new("list-view"),
            vec!["First", "Second", "Third", "Fourth", "Fifth"],
            1,
        );
        scene.add_view_to_parent(list, &wrapper.name);
        wrapper.visible = false;
        scene.add_view_to_parent(wrapper, &tabbed_panel.name);
    }
    {
        let mut panel = View {
            name: INPUTS_PANEL.clone(),
            draw: Some(draw_std_panel),
            h_flex: Flex::Resize,
            v_flex: Flex::Resize,
            layout: Some(layout_std_panel),
            ..Default::default()
        };
        scene.add_view_to_parent(
            make_text_input("textinput", "input").position_at(10, 10),
            &panel.name,
        );
        panel.visible = false;
        scene.add_view_to_parent(panel, &tabbed_panel.name);
    }
    {
        let mut panel = View {
            name: THEMES_PANEL.clone(),
            layout: Some(layout_vbox),
            draw: Some(draw_std_panel),
            h_flex: Flex::Resize,
            v_flex: Flex::Resize,
            ..Default::default()
        };

        scene.add_view_to_parent(
            make_label("themes-label", "Themes").position_at(30, 90),
            &panel.name,
        );
        scene.add_view_to_parent(
            make_button(&ViewId::new("light-theme"), "Light"),
            &panel.name,
        );
        scene.add_view_to_parent(make_button(&ViewId::new("dark-theme"), "Dark"), &panel.name);
        scene.add_view_to_parent(
            make_button(&ViewId::new("colorful-theme"), "Colorful"),
            &panel.name,
        );
        panel.visible = false;
        scene.add_view_to_parent(panel, &tabbed_panel.name);
    }

    scene.add_view_to_root(tabbed_panel);

    {
        let mut font_buttons = View {
            name: ViewId::new("font_buttons"),
            bounds: Bounds::new(30, 200, 200, 30),
            layout: Some(layout_hbox),
            h_flex: Intrinsic,
            v_flex: Intrinsic,
            draw: Some(draw_std_panel),
            ..Default::default()
        };
        scene.add_view_to_parent(make_button(SMALL_FONT_BUTTON, "Small"), &font_buttons.name);
        scene.add_view_to_parent(
            make_button(MEDIUM_FONT_BUTTON, "Medium"),
            &font_buttons.name,
        );
        scene.add_view_to_parent(make_button(LARGE_FONT_BUTTON, "Large"), &font_buttons.name);
        scene.add_view_to_root(font_buttons);
    }

    if let Some(state) = scene.get_view_state::<SelectOneOfState>(TABBED_PANEL) {
        state.selected = 2;
    }

    scene
}
fn make_vbox_test() -> Scene {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));
    let parent_id: ViewId = "parent".into();
    let parent_view = View {
        name: parent_id.clone(),
        title: "parent".into(),
        padding: Insets::new_same(10),
        bounds: Bounds::new(0, 0, 100, 100),
        h_flex: Resize,
        v_flex: Resize,
        h_align: Start,
        v_align: Start,
        layout: Some(layout_hbox),
        draw: Some(draw_std_panel),
        ..Default::default()
    };
    {
        let child1_id: ViewId = "child1".into();
        let mut child = make_button(&child1_id, "ch1");
        child.h_align = Align::Start;
        child.v_align = Start;
        scene.add_view_to_parent(child, &parent_id);

        let child2_id: ViewId = "child2".into();
        let mut child = make_button(&child2_id, "ch2");
        child.h_align = Align::Center;
        child.v_align = Center;
        scene.add_view_to_parent(child, &parent_id);

        let child3_id: ViewId = "child3".into();
        let mut child = make_button(&child3_id, "ch3");
        child.h_align = Align::End;
        child.v_align = Align::End;
        scene.add_view_to_parent(child, &parent_id);
    }

    let child_box = View {
        name: ViewId::new("foo"),
        padding: Insets::new_same(5),
        layout: Some(layout_vbox),
        draw: Some(draw_std_panel),
        bounds: Bounds::new(0, 0, 100, 100),
        h_flex: Intrinsic,
        v_flex: Resize,
        h_align: Center,
        ..Default::default()
    };
    {
        let child1_id: ViewId = "child1a".into();
        let mut child = make_button(&child1_id, "ch1");
        child.h_align = Align::Start;
        child.v_align = Start;
        scene.add_view_to_parent(child, &child_box.name);

        let child2_id: ViewId = "child2a".into();
        let mut child = make_button(&child2_id, "ch2");
        child.h_align = Align::Center;
        child.v_align = Center;
        scene.add_view_to_parent(child, &child_box.name);

        let child3_id: ViewId = "child3a".into();
        let mut child = make_button(&child3_id, "ch3");
        child.h_align = Align::End;
        child.v_align = Align::End;
        scene.add_view_to_parent(child, &child_box.name);
    }
    scene.add_view_to_parent(child_box, &parent_id);

    // let child4_id: ViewId = "child4".into();
    // scene.add_view_to_parent(
    //     View {
    //         name: child4_id.clone(),
    //         title: "ch4".into(),
    //         h_flex: Flex::Resize,
    //         v_flex: Flex::Resize,
    //         layout: Some(layout_std_panel),
    //         ..Default::default()
    //     },
    //     &parent_id,
    // );

    scene.add_view_to_parent(parent_view, &scene.root_id());
    scene
}

fn make_column(name: &'static str) -> View {
    let panel = View {
        name: ViewId::new(name),
        draw: Some(draw_std_panel),
        h_flex: Flex::Resize,
        v_flex: Flex::Resize,
        h_align: Align::Center,
        v_align: Align::Start,
        layout: Some(layout_vbox),
        ..Default::default()
    };
    panel
}

fn make_row(name: &'static str) -> View {
    View {
        name: ViewId::new(name),
        draw: Some(draw_std_panel),
        h_flex: Flex::Resize,
        v_flex: Flex::Resize,
        layout: Some(layout_hbox),
        ..Default::default()
    }
}

fn main() -> Result<(), std::convert::Infallible> {
    env_logger::Builder::new()
        .target(Target::Stdout) // <-- redirects to stdout
        .filter(None, LevelFilter::Info)
        .init();

    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));

    let mut scene = make_scene();
    // let mut scene = make_vbox_test();
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
        let mut ctx = EmbeddedDrawingContext::new(&mut display);
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
                    let evt: EventType = keydown_to_char(keycode, keymod);
                    if let Some(result) = event_at_focused(&mut scene, &evt) {
                        println!("got input from {:?}", result);
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
                SimulatorEvent::MouseWheel {
                    scroll_delta,
                    direction,
                } => {
                    info!("mouse wheel {scroll_delta:?} {direction:?}");
                    if let Some(result) = event_at_focused(
                        &mut scene,
                        &EventType::Scroll(scroll_delta.x, scroll_delta.y),
                    ) {
                        println!("got input from {:?}", result);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn keydown_to_char(keycode: Keycode, keymod: Mod) -> EventType {
    println!("keycode as number {}", keycode.into_i32());
    let ch = keycode.into_i32();
    if ch <= 0 {
        return EventType::Unknown;
    }
    let shifted = keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD);

    if let Some(ch) = char::from_u32(ch as u32) {
        if ch.is_alphabetic() {
            return if shifted {
                EventType::Keyboard(ch.to_ascii_uppercase() as u8)
            } else {
                EventType::Keyboard(ch.to_ascii_lowercase() as u8)
            };
        }
        if ch.is_ascii_graphic() {
            return EventType::Keyboard(ch as u8);
        }
    }
    match keycode {
        Keycode::Backspace => EventType::KeyboardAction(KeyboardAction::Backspace),
        Keycode::LEFT => EventType::KeyboardAction(KeyboardAction::Left),
        Keycode::RIGHT => EventType::KeyboardAction(KeyboardAction::Right),
        Keycode::UP => EventType::KeyboardAction(KeyboardAction::Up),
        Keycode::DOWN => EventType::KeyboardAction(KeyboardAction::Down),
        Keycode::SPACE => EventType::Keyboard(b' '),
        _ => {
            println!("not supported: {keycode}");
            return EventType::Unknown;
        }
    }
}

fn handle_events(result: EventResult, scene: &mut Scene, theme: &mut Theme) {
    let (name, action) = result;
    println!("result of event {:?} from {name}", action);
    if name == *SMALL_FONT_BUTTON {
        theme.font = FONT_5X7;
        theme.bold_font = FONT_5X7;
        scene.mark_layout_dirty();
    }
    if name == *MEDIUM_FONT_BUTTON {
        theme.font = FONT_6X10;
        theme.bold_font = FONT_6X10;
        scene.mark_layout_dirty();
    }
    if name == *LARGE_FONT_BUTTON {
        theme.font = FONT_7X13;
        theme.bold_font = FONT_7X13_BOLD;
        scene.mark_layout_dirty();
    }
    if name.as_str() == "light-theme" {
        theme.bg = Rgb565::WHITE;
        theme.fg = Rgb565::BLACK;
        theme.panel_bg = Rgb565::CSS_LIGHT_GRAY;
        theme.selected_bg = Rgb565::CSS_CORNFLOWER_BLUE;
        theme.selected_fg = Rgb565::WHITE;
        scene.mark_dirty_all();
    }
    if name.as_str() == "dark-theme" {
        theme.bg = Rgb565::from(Rgb888::new(50, 50, 50));
        theme.fg = Rgb565::WHITE;
        theme.panel_bg = Rgb565::BLACK;
        theme.selected_bg = Rgb565::CSS_DARK_BLUE;
        theme.selected_fg = Rgb565::WHITE;
        scene.mark_dirty_all();
    }
    if name.as_str() == "colorful-theme" {
        theme.bg = Rgb565::CSS_MISTY_ROSE;
        theme.fg = Rgb565::CSS_DARK_BLUE;
        theme.panel_bg = Rgb565::CSS_ANTIQUE_WHITE;
        theme.selected_bg = Rgb565::CSS_DARK_MAGENTA;
        theme.selected_fg = Rgb565::CSS_LIGHT_YELLOW;
        scene.mark_dirty_all();
    }
    if name == *POPUP_BUTTON {
        let menu =
            make_list_view(POPUP_MENU, vec!["Item 1", "Item 2", "Item 3"], 0).position_at(50, 50);
        scene.set_focused(&menu.name);
        scene.add_view_to_root(menu);
    }
    if name == *POPUP_MENU {
        scene.remove_view(POPUP_MENU);
    }

    if name.as_str() == "tabs" {
        match action {
            Action::Command(cmd) => {
                for kid in scene.get_children_ids(TABBED_PANEL) {
                    if kid.as_str() != "tabs" {
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
