use crate::geom::Bounds;
use crate::gfx::{DrawingContext, TextStyle};
use crate::view::{Align, View, ViewId};
use crate::{Action, DrawEvent, EventType, GuiEvent};
use log::info;

fn draw_text_input(e: &mut DrawEvent) {
    e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
    let style = TextStyle::new(&e.theme.font, &e.theme.fg).with_halign(Align::Start);
    e.ctx.fill_text(&e.view.bounds, &e.view.title, &style);

    if let Some(focused) = e.focused {
        if focused == &e.view.name {
            e.ctx.stroke_rect(&e.view.bounds.contract(2), &e.theme.fg);
            let n = e.view.title.len() as i32;
            let w = e.theme.font.character_size.width as i32;
            let h = e.theme.font.character_size.height as i32;
            e.ctx.fill_rect(
                &Bounds::new(e.view.bounds.position.x+ n * w + 5, e.view.bounds.position.y+ 5, 2, h + 4),
                &e.theme.fg,
            );
        }
    }
}

fn input_text_input(event: &mut GuiEvent) -> Option<Action> {
    info!("text input got event {:?}", event.event_type);
    match &event.event_type {
        EventType::Keyboard(key) => {
            if let Some(view) = event.scene.get_view_mut(event.target) {
                match *key {
                    8 => {
                        if view.title.len() > 0 {
                            view.title.remove(view.title.len() - 1);
                        }
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
        EventType::KeyboardAction(act) => {
            if let Some(view) = event.scene.get_view_mut(event.target) {
                if view.title.len() > 0 {
                    view.title.remove(view.title.len() - 1);
                    event.scene.mark_dirty_view(event.target);
                }
            }
        }
        EventType::Tap(_pt) => {
            event.scene.set_focused(event.target);
        }
        _ => {}
    }
    None
}

pub fn make_text_input(name: &'static str, title: &str) -> View {
    View {
        name: ViewId::new(name),
        title: title.into(),
        bounds: Bounds::new(0,0,100,30),
        visible: true,
        state: None,
        input: Some(input_text_input),
        layout: Some(|e| {
            // if let Some(view) = e.scene.get_view_mut(e.target) {
            //     view.bounds = util::calc_bounds(view.bounds, e.theme.bold_font, &view.title);
            // }
        }),
        draw: Some(draw_text_input),
        .. Default::default()
    }
}
