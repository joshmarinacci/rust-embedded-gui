use crate::HAlign::Left;
use crate::geom::Bounds;
use crate::view::View;
use crate::{
    Action, DrawEvent, DrawingContext, EventType, GuiEvent, HAlign, LayoutEvent, TextStyle, Theme,
    VAlign, util,
};
use alloc::string::ToString;
use log::info;

fn draw_button(e: &mut DrawEvent) {
    e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
    if let Some(focused) = e.focused {
        if focused == &e.view.name {
            e.ctx.stroke_rect(&e.view.bounds.contract(2), &e.theme.fg);
        }
    }
    e.ctx.fill_text(
        &e.view.bounds,
        &e.view.title,
        &TextStyle {
            font: &e.theme.bold_font,
            halign: HAlign::Center,
            color: &e.theme.fg,
            underline: false,
            valign: VAlign::Center,
        },
    );
}

pub fn make_button(name: &str, title: &str) -> View {
    View {
        name: name.to_string(),
        title: title.to_string(),
        bounds: Bounds {
            x: 0,
            y: 0,
            w: 80,
            h: 30,
        },
        visible: true,
        state: None,
        input: Some(|e| {
            if let EventType::Tap(_pt) = &e.event_type {
                e.scene.set_focused(e.target);
                e.scene.mark_dirty_view(e.target);
                return Some(Action::Generic);
            }
            None
        }),
        layout: Some(|e| {
            if let Some(view) = e.scene.get_view_mut(e.target) {
                view.bounds = util::calc_bounds(view.bounds, e.theme.bold_font, &view.title);
            }
        }),
        draw: Some(draw_button),
    }
}

pub fn make_label(name: &str, title: &str) -> View {
    View {
        name: name.into(),
        title: title.into(),
        bounds: Bounds {
            x: 0,
            y: 0,
            w: 100,
            h: 30,
        },
        visible: true,
        state: None,
        input: None,
        draw: Some(|e| {
            let style = TextStyle::new(&e.theme.font, &e.theme.fg);
            e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
        }),
        layout: Some(|e| {
            if let Some(view) = e.scene.get_view_mut(e.target) {
                view.bounds = util::calc_bounds(view.bounds, e.theme.bold_font, &view.title);
            }
        }),
    }
}

fn draw_text_input(e: &mut DrawEvent) {
    e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
    if let Some(focused) = e.focused {
        if focused == &e.view.name {
            e.ctx.stroke_rect(&e.view.bounds.contract(2), &e.theme.fg);
        }
    }
    let style = TextStyle::new(&e.theme.font, &e.theme.fg).with_halign(HAlign::Left);
    e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
}

fn input_text_input(event: &mut GuiEvent) -> Option<Action> {
    info!("text input got event {:?}", event.event_type);
    match &event.event_type {
        EventType::Keyboard(key) => {
            if let Some(view) = event.scene.get_view_mut(event.target) {
                match *key {
                    8 => {
                        view.title.remove(view.title.len() - 1);
                    }
                    13 => {
                        info!("doing return");
                        return Some(Action::Command("Completed".into()));
                    }
                    _ => {
                        view.title.push(*key as char);
                    }
                }
                info!("done");
            }
            event.scene.mark_dirty_view(event.target);
        }
        EventType::Tap(_pt) => {
            event.scene.set_focused(event.target);
        }
        _ => {}
    }
    None
}
pub fn make_text_input(name: &str, title: &str) -> View {
    View {
        name: name.into(),
        title: title.into(),
        bounds: Bounds {
            x: 0,
            y: 0,
            w: 100,
            h: 30,
        },
        visible: true,
        state: None,
        input: Some(input_text_input),
        layout: Some(|e| {
            // if let Some(view) = e.scene.get_view_mut(e.target) {
            //     view.bounds = util::calc_bounds(view.bounds, e.theme.bold_font, &view.title);
            // }
        }),
        draw: Some(draw_text_input),
    }
}
