use alloc::string::ToString;
use alloc::vec;
use log::info;
use crate::geom::Bounds;
use crate::{DrawingContext, EventType, GuiEvent, HAlign, Theme, View};

fn draw_panel<C>(view: &mut View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    ctx.fill_rect(&view.bounds, &theme.bg);
    ctx.stroke_rect(&view.bounds, &theme.fg);
}
pub fn make_panel<C>(name: &str, bounds: Bounds) -> View<C> {
    View {
        name: name.into(),
        title: name.into(),
        bounds,
        visible: true,
        children: vec![],
        draw: Some(draw_panel),
        input: None,
        state: None,
        layout: None,
    }
}



fn draw_button<C>(view: &mut View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    ctx.fill_rect(&view.bounds, &theme.bg);
    ctx.stroke_rect(&view.bounds, &theme.fg);
    ctx.fill_text(&view.bounds, &view.title, &theme.fg, &HAlign::Center);
}

fn input_button<C>(event:&mut GuiEvent<C>) {
    info!("button got input {:?} {:?}", event.target, event.event_type);
    match &event.event_type {
        EventType::Tap(pt) => {
            info!("tapped on text input");
            event.scene.focused = Some(event.target.into());
        }
        _ => {}
    }
}
pub fn make_button<C>(name: &str, title: &str) -> View<C> {
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
        children: vec![],
        draw: Some(draw_button),
        input:Some(input_button),
        state: None,
        layout: None,
    }
}

fn draw_label<C>(view: &mut View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    ctx.fill_text(&view.bounds, &view.title, &theme.fg, &HAlign::Left);
}
pub fn make_label<C>(name: &str, title: &str) -> View<C> {
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
        children: vec![],
        draw: Some(draw_label),
        input: None,
        state: None,
        layout: None,
    }
}
fn draw_text_input<C>(view: &mut View<C>, ctx: &mut dyn DrawingContext<C>, theme: &Theme<C>) {
    ctx.fill_rect(&view.bounds, &theme.bg);
    ctx.stroke_rect(&view.bounds, &theme.fg);
    ctx.fill_text(&view.bounds, &view.title, &theme.fg, &HAlign::Left);
    // if view.focused {
    //     let cursor = Bounds {
    //         x: view.bounds.x + 20,
    //         y: view.bounds.y + 2,
    //         w: 2,
    //         h: view.bounds.h - 4,
    //     };
    //     ctx.fillRect(&cursor, &theme.fg);
    // }
}

fn input_text_input<C>(event:&mut GuiEvent<C>) {
    info!("text input got event {:?}",event.event_type);
    match &event.event_type {
        EventType::Keyboard(key) => {
            if let Some(view) = event.scene.get_view_mut(event.target) {
                if *key == 8 {
                    view.title.remove(view.title.len()-1);
                } else {
                    view.title.push(*key as char);
                }
            }
            event.scene.dirty = true;
        }
        EventType::Tap(pt) => {
            info!("tapped on text input");
            event.scene.focused = Some(event.target.into());
        }
        _ => {

        }
    }
}
pub fn make_text_input<C>(name:&str, title: &str) -> View<C> {
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
        children: vec![],
        draw: Some(draw_text_input),
        input: Some(input_text_input),
        state: None,
        layout: None,
    }
}
