use crate::geom::Bounds;
use crate::view::View;
use crate::{
    Action, DrawEvent, DrawingContext, EventType, GuiEvent, HAlign, TextStyle, Theme, VAlign,
};
use alloc::string::ToString;
use log::info;

pub fn make_panel<C, F>(name: &str, bounds: Bounds) -> View<C, F> {
    View {
        name: name.into(),
        title: name.into(),
        bounds,
        visible: true,
        draw: Some(|e| {
            e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
            e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
        }),
        input: None,
        state: None,
        layout: None,
    }
}

fn draw_button<C, F>(e: &mut DrawEvent<C, F>) {
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

fn input_button<C, F>(event: &mut GuiEvent<C, F>) -> Option<Action> {
    // warn!("button got input {:?} {:?}", event.target, event.event_type);
    if let EventType::Tap(_pt) = &event.event_type {
        event.scene.set_focused(event.target);
        event.scene.mark_dirty_view(event.target);
        return Some(Action::Generic);
    }
    None
}
pub fn make_button<C, F>(name: &str, title: &str) -> View<C, F> {
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
        draw: Some(draw_button),
        input: Some(input_button),
        state: None,
        layout: None,
    }
}

pub fn make_label<C, F>(name: &str, title: &str) -> View<C, F> {
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
        draw: Some(|e| {
            let style = TextStyle::new(&e.theme.font, &e.theme.fg);
            e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
        }),
        input: None,
        state: None,
        layout: None,
    }
}

fn draw_text_input<C, F>(e: &mut DrawEvent<C, F>) {
    e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
    if let Some(focused) = e.focused {
        if focused == &e.view.name {
            e.ctx.stroke_rect(&e.view.bounds.contract(2), &e.theme.fg);
        }
    }
    let style = TextStyle::new(&e.theme.font, &e.theme.fg);
    e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);
}

fn input_text_input<C, F>(event: &mut GuiEvent<C, F>) -> Option<Action> {
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
pub fn make_text_input<C, F>(name: &str, title: &str) -> View<C, F> {
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
        draw: Some(draw_text_input),
        input: Some(input_text_input),
        state: None,
        layout: None,
    }
}
