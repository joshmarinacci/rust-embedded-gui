use crate::geom::Bounds;
use crate::gfx::TextStyle;
use crate::view::{Align, View, ViewId};
use crate::{Action, DrawEvent, EventType, GuiEvent, KeyboardAction};
use alloc::boxed::Box;
use alloc::string::String;
use core::str::Chars;
use log::info;


pub struct TextInputState {
    cursor: usize,
    text: String,
}

impl TextInputState {
    fn cursor_back(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }
    fn cursor_forward(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor += 1;
        }
    }
    fn delete_back(&mut self) {
        if self.cursor > 0 && self.cursor <= self.text.len() {
            self.cursor -= 1;
            self.text.remove(self.cursor);
        }
    }
    fn delete_forward(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }
    fn insert_char(&mut self, key: char) {
        self.text.insert(self.cursor, key);
        self.cursor += 1;
    }
}

fn draw_text_input(e: &mut DrawEvent) {
    e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
    let style = TextStyle::new(&e.theme.font, &e.theme.fg).with_halign(Align::Start);

    let bounds = e.view.bounds.clone();
    if let Some(state) = e.view.get_state::<TextInputState>() {
        e.ctx.fill_text(&bounds, &state.text, &style);
    }

    if let Some(focused) = e.focused {
        if focused == &e.view.name {
            e.ctx.stroke_rect(&e.view.bounds.contract(2), &e.theme.fg);
            if let Some(state) = e.view.get_state::<TextInputState>() {
                let n = state.cursor as i32;
                let w = e.theme.font.character_size.width as i32;
                let h = e.theme.font.character_size.height as i32;
                e.ctx.fill_rect(
                    &Bounds::new(
                        e.view.bounds.position.x + n * w + 5,
                        e.view.bounds.position.y + 5,
                        1,
                        h + 4,
                    ),
                    &e.theme.selected_bg,
                );
            }
        }
    }
}

fn input_text_input(event: &mut GuiEvent) -> Option<Action> {
    info!("text input got event {:?}", event.event_type);
    match &event.event_type {
        EventType::Keyboard(key) => {
            if let Some(state) = event.scene.get_view_state::<TextInputState>(event.target) {
                match *key {
                    8 => {
                        state.delete_back();
                    }
                    13 => {
                        info!("doing return");
                        return Some(Action::Command("Completed".into()));
                    }
                    _ => {
                        state.insert_char(*key as char);
                    }
                }
                info!("done");
            }
            event.scene.mark_dirty_view(event.target);
        }
        EventType::KeyboardAction(act) => {
            if let Some(state) = event.scene.get_view_state::<TextInputState>(event.target) {
                match act {
                    KeyboardAction::Left => state.cursor_back(),
                    KeyboardAction::Right => state.cursor_forward(),
                    KeyboardAction::Up => {}
                    KeyboardAction::Down => {}
                    KeyboardAction::Backspace => state.delete_back(),
                    KeyboardAction::Delete => state.delete_forward(),
                    KeyboardAction::Return => {}
                }
                event.scene.mark_dirty_view(event.target);
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
        bounds: Bounds::new(0, 0, 100, 30),
        visible: true,
        state: Some(Box::new(TextInputState {
            text: title.into(),
            cursor: title.len(),
        })),
        input: Some(input_text_input),
        layout: Some(|_e| {
            // if let Some(view) = e.scene.get_view_mut(e.target) {
            //     view.bounds = util::calc_bounds(view.bounds, e.theme.bold_font, &view.title);
            // }
        }),
        draw: Some(draw_text_input),
        ..Default::default()
    }
}
