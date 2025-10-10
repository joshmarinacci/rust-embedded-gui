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
use iris_ui::button::make_button;
use iris_ui::geom::{Bounds, Insets, Point as GPoint};
use iris_ui::scene::{click_at, draw_scene, event_at_focused, layout_scene, Scene};
use iris_ui::toggle_button::make_toggle_button;
use iris_ui::toggle_group::{layout_toggle_group, make_toggle_group, SelectOneOfState};
use iris_ui::{util, Theme};
use std::convert::Into;

use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::{Keycode, Mod};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use env_logger::fmt::style::Color::Rgb;
use env_logger::Target;
use iris_ui::device::EmbeddedDrawingContext;
use iris_ui::grid::{make_grid_panel, GridLayoutState, LayoutConstraint};
use iris_ui::input::{InputEvent, InputResult, OutputAction, TextAction};
use iris_ui::label::make_label;
use iris_ui::layouts::{layout_hbox, layout_std_panel, layout_vbox};
use iris_ui::list_view::make_list_view;
use iris_ui::panel::{draw_borderless_panel, draw_std_panel, PanelState};
use iris_ui::tabbed_panel::{make_tabbed_panel, LayoutPanelState};
use iris_ui::text_input::make_text_input;
use iris_ui::util::hex_str_to_rgb565;
use iris_ui::view::Align::{Center, Start};
use iris_ui::view::Flex::{Intrinsic, Resize};
use iris_ui::view::{Align, Flex, View, ViewId};
use log::{info, LevelFilter};

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

    let tab_names = vec!["buttons", "layouts", "lists", "inputs", "themes"];
    let mut tabbed_panel: View = make_tabbed_panel(&TABBED_PANEL, tab_names, 0, &mut scene);
    if let Some(state) = tabbed_panel.get_state::<LayoutPanelState>() {
        state.register_panel("buttons", BUTTONS_PANEL);
        state.register_panel("layouts", LAYOUT_PANEL);
        state.register_panel("lists", LISTS_PANEL);
        state.register_panel("inputs", INPUTS_PANEL);
        state.register_panel("themes", THEMES_PANEL);
    }

    {
        let mut grid = make_grid_panel(BUTTONS_PANEL)
            .with_draw_fn(Some(draw_borderless_panel))
            .with_padding(Insets::new_same(10));
        grid.h_flex = Resize;
        grid.v_flex = Resize;
        let mut grid_layout = GridLayoutState::new_row_column(3, 30, 2, 100);
        grid_layout.debug = false;

        let label1 = make_label("label1", "A Label");
        grid_layout.place_at_row_column(&label1.name, 0, 0);
        scene.add_view_to_parent(label1, &grid.name);

        let button1 = make_button(&ViewId::new("button1"), "Action Button");
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
            draw: Some(draw_borderless_panel),
            padding: Insets::new_same(5),
            h_flex: Resize,
            v_flex: Resize,
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
            draw: Some(draw_borderless_panel),
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
        wrapper.hide();
        scene.add_view_to_parent(wrapper, &tabbed_panel.name);
    }
    {
        let mut panel = View {
            name: INPUTS_PANEL.clone(),
            draw: Some(draw_borderless_panel),
            h_flex: Resize,
            v_flex: Resize,
            layout: Some(layout_std_panel),
            ..Default::default()
        };
        scene.add_view_to_parent(
            make_text_input("text input", "input").position_at(10, 10),
            &panel.name,
        );
        panel.hide();
        scene.add_view_to_parent(panel, &tabbed_panel.name);
    }
    {
        let panel = make_column(THEMES_PANEL.as_str())
            .with_draw_fn(Some(draw_borderless_panel))
            .with_padding(Insets::new_same(10))
            .with_visible(false)
            ;
        let themes_list_id = ViewId::new("themes-list");
        let themes = make_list_view(
            &themes_list_id,
            vec!["Light", "Dark", "Ice Cream", "Minty Fresh", "Amber"],
            0,
        );
        scene.add_view_to_parent(themes, &panel.name);
        scene.add_view_to_parent(panel, &tabbed_panel.name);
    }
    scene.add_view_to_root(tabbed_panel);

    {
        let font_buttons = View {
            name: ViewId::new("font_buttons"),
            bounds: Bounds::new(30, 200, 200, 30),
            state: Some(Box::new(PanelState {
                border_visible: true,
                gap: 5,
            })),
            layout: Some(layout_hbox),
            h_flex: Intrinsic,
            v_flex: Intrinsic,
            draw: Some(draw_std_panel),
            ..Default::default()
        }.with_padding(Insets::new_same(5));
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
    View {
        name: ViewId::new(name),
        draw: Some(draw_std_panel),
        h_flex: Resize,
        v_flex: Resize,
        h_align: Center,
        v_align: Start,
        layout: Some(layout_vbox),
        ..Default::default()
    }
}

fn make_row(name: &'static str) -> View {
    View {
        name: ViewId::new(name),
        draw: Some(draw_std_panel),
        h_flex: Resize,
        v_flex: Resize,
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
    copy_theme_colors(&mut theme, &LIGHT_THEME);

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
                    let act: TextAction = keydown_to_char(keycode, keymod);
                    let evt = InputEvent::Text(act);
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
                        &InputEvent::Scroll(GPoint::new(scroll_delta.x, scroll_delta.y)),
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

fn keydown_to_char(keycode: Keycode, keymod: Mod) -> TextAction {
    println!("keycode as number {}", keycode.into_i32());
    let ch = keycode.into_i32();
    if ch <= 0 {
        return TextAction::Unknown;
    }
    let shifted = keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD);
    let controlled = keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD);

    if let Some(ch) = char::from_u32(ch as u32) {
        if ch == 'd' && controlled {
            return TextAction::ForwardDelete;
        }
        if ch.is_alphabetic() {
            return if shifted {
                TextAction::TypedAscii(ch.to_ascii_uppercase() as u8)
            } else {
                TextAction::TypedAscii(ch.to_ascii_lowercase() as u8)
            };
        }
        if ch.is_ascii_graphic() {
            return TextAction::TypedAscii(ch as u8);
        }
    }
    match keycode {
        Keycode::Backspace => TextAction::BackDelete,
        Keycode::LEFT => TextAction::Left,
        Keycode::RIGHT => TextAction::Right,
        Keycode::UP => TextAction::Up,
        Keycode::DOWN => TextAction::Down,
        Keycode::SPACE => TextAction::TypedAscii(b' '),
        _ => {
            println!("not supported: {keycode}");
            return TextAction::Unknown;
        }
    }
}

fn handle_events(result: InputResult, scene: &mut Scene, theme: &mut Theme) {
    println!("result of event {:?} from {}", result.input, result.source);
    if result.source == *SMALL_FONT_BUTTON {
        theme.font = FONT_5X7;
        theme.bold_font = FONT_5X7;
        scene.mark_layout_dirty();
    }
    if result.source == *MEDIUM_FONT_BUTTON {
        theme.font = FONT_6X10;
        theme.bold_font = FONT_6X10;
        scene.mark_layout_dirty();
    }
    if result.source == *LARGE_FONT_BUTTON {
        theme.font = FONT_7X13;
        theme.bold_font = FONT_7X13_BOLD;
        scene.mark_layout_dirty();
    }
    if result.source.as_str() == "themes-list" {
        match &result.action {
            Some(OutputAction::Command(cmd)) => {
                match cmd.as_str() {
                    "Dark" => copy_theme_colors(theme, &DARK_THEME),
                    "Light" => copy_theme_colors(theme, &LIGHT_THEME),
                    "Ice Cream" => copy_theme_colors(theme, &ICE_CREAM_THEME),
                    "Minty Fresh" => copy_theme_colors(theme, &MINTY_FRESH),
                    "Amber" => copy_theme_colors(theme, &AMBER),
                    _ => {}
                }
                scene.mark_dirty_all();
            }
            _ => {}
        }
    }
    if result.source == *POPUP_BUTTON {
        let menu =
            make_list_view(POPUP_MENU, vec!["Item 1", "Item 2", "Item 3"], 0).position_at(50, 50);
        scene.set_focused(&menu.name);
        scene.add_view_to_root(menu);
    }
    if result.source == *POPUP_MENU {
        scene.remove_view(POPUP_MENU);
    }
}

const LIGHT_THEME: Theme = Theme {
    bg: Rgb565::WHITE,
    fg: Rgb565::BLACK,
    panel_bg: Rgb565::CSS_LIGHT_GRAY,
    selected_bg: hex_str_to_rgb565("#4488ff"),
    selected_fg: Rgb565::WHITE,
    font: FONT_7X13,
    bold_font: FONT_7X13_BOLD,
};
const DARK_THEME: Theme = Theme {
    bg: hex_str_to_rgb565("#222222"),
    fg: hex_str_to_rgb565("#999999"),
    panel_bg: Rgb565::BLACK,
    selected_bg: hex_str_to_rgb565("#000088"),
    selected_fg: hex_str_to_rgb565("#3366ff"),
    font: FONT_7X13,
    bold_font: FONT_7X13_BOLD,
};

//https://lospec.com/palette-list/ice-cream-gb
const ICE_CREAM_THEME: Theme = Theme {
    bg: hex_str_to_rgb565("fff6d3"),
    fg: hex_str_to_rgb565("#7c3f58"),
    panel_bg: hex_str_to_rgb565("fff6d3"),
    selected_bg: hex_str_to_rgb565("#eb6b6f"),
    selected_fg: hex_str_to_rgb565("#fff6d3"),
    font: FONT_7X13,
    bold_font: FONT_7X13_BOLD,
};
//https://lospec.com/palette-list/minty-fresh
const MINTY_FRESH: Theme = Theme {
    bg: hex_str_to_rgb565("#856d52"),
    fg: hex_str_to_rgb565("#fbffe0"),
    panel_bg: hex_str_to_rgb565("#40332f"),
    selected_bg: hex_str_to_rgb565("#95c798"),
    selected_fg: hex_str_to_rgb565("#40332f"),
    font: FONT_7X13,
    bold_font: FONT_7X13_BOLD,
};
//https://lospec.com/palette-list/amber-crtgb
const AMBER: Theme = Theme {
    bg: hex_str_to_rgb565("#0d0405"),
    fg: hex_str_to_rgb565("#d35600"),
    panel_bg: hex_str_to_rgb565("#0d0405"),
    selected_bg: hex_str_to_rgb565("#fed018"),
    selected_fg: hex_str_to_rgb565("#5e1210"),
    font: FONT_7X13,
    bold_font: FONT_7X13_BOLD,
};

fn copy_theme_colors(theme: &mut Theme, new: &Theme) {
    theme.bg = new.bg;
    theme.panel_bg = new.panel_bg;
    theme.selected_bg = new.selected_bg;
    theme.fg = new.fg;
    theme.selected_fg = new.selected_fg;
}
