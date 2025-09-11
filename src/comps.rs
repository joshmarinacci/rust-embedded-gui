use alloc::string::ToString;
use alloc::vec;
use log::{info, warn};
use crate::geom::Bounds;
use crate::{Action, DrawEvent, DrawingContext, EventType, GuiEvent, HAlign, Theme, View};

fn draw_panel<C, F>(view: &mut View<C, F>, ctx: &mut dyn DrawingContext<C, F>, theme: &Theme<C, F>) {
    ctx.fill_rect(&view.bounds, &theme.bg);
    ctx.stroke_rect(&view.bounds, &theme.fg);
}
pub fn make_panel<C, F>(name: &str, bounds: Bounds) -> View<C, F> {
    View {
        name: name.into(),
        title: name.into(),
        bounds,
        visible: true,
        draw: Some(draw_panel),
        draw2: None,
        input: None,
        state: None,
        layout: None,
    }
}



fn draw_button<C, F>(e:&mut DrawEvent<C, F>) {
    e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
    if let Some(focused) = e.focused {
        if focused == &e.view.name {
            e.ctx.stroke_rect(&e.view.bounds.contract(2), &e.theme.fg);
        }
    }
    e.ctx.fill_text(&e.view.bounds, &e.view.title, &e.theme.fg, &HAlign::Center);
}

fn input_button<C, F>(event:&mut GuiEvent<C, F>) -> Option<Action> {
    // warn!("button got input {:?} {:?}", event.target, event.event_type);
    match &event.event_type {
        EventType::Tap(pt) => {
            event.scene.set_focused(event.target);
            event.scene.mark_dirty_view(event.target);
            return Some(Action::Generic)
        }
        _ => {}
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
        draw: None,
        draw2: Some(draw_button),
        input:Some(input_button),
        state: None,
        layout: None,
    }
}

fn draw_label<C, F>(view: &mut View<C, F>, ctx: &mut dyn DrawingContext<C, F>, theme: &Theme<C, F>) {
    ctx.fill_text(&view.bounds, &view.title, &theme.fg, &HAlign::Left);
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
        draw: Some(draw_label),
        draw2: None,
        input: None,
        state: None,
        layout: None,
    }
}

fn draw_text_input<C, F>(e:&mut DrawEvent<C, F>) {
    e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
    if let Some(focused) = e.focused {
        if focused == &e.view.name {
            e.ctx.stroke_rect(&e.view.bounds.contract(2), &e.theme.fg);
        }
    }
    e.ctx.fill_text(&e.view.bounds, &e.view.title, &e.theme.fg, &HAlign::Left);
}

fn input_text_input<C, F>(event:&mut GuiEvent<C, F>) -> Option<Action> {
    info!("text input got event {:?}",event.event_type);
    match &event.event_type {
        EventType::Keyboard(key) => {
            if let Some(view) = event.scene.get_view_mut(event.target) {
                match *key {
                    8 => {
                        view.title.remove(view.title.len()-1);
                    },
                    13 => {
                        info!("doing return");
                        return Some(Action::Command("Completed".into()));
                    },
                    _ => {
                        view.title.push(*key as char);
                    }
                }
                info!("done");
            }
            event.scene.mark_dirty_view(event.target);
        }
        EventType::Tap(pt) => {
            event.scene.set_focused(event.target);
        }
        _ => {

        }
    }
    None
}
pub fn make_text_input<C, F>(name:&str, title: &str) -> View<C, F> {
    View {
        name: name.into(),
        title: title.into(),
        bounds:Bounds {
            x: 0,
            y: 0,
            w: 100,
            h: 30,
        },
        visible: true,
        draw2: Some(draw_text_input),
        draw: None,
        input: Some(input_text_input),
        state: None,
        layout: None,
    }
}
