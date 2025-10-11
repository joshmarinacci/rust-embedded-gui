use crate::DrawEvent;

pub struct PanelState {
    pub gap: i32,
    pub border_visible: bool,
}

pub fn draw_std_panel(e: &mut DrawEvent) {
    let bounds = e.view.bounds;
    e.ctx.fill_rect(&bounds, &e.theme.panel.fill);
    if let Some(state) = e.view.get_state::<PanelState>() {
        if state.border_visible {
            e.ctx.stroke_rect(&bounds, &e.theme.panel.text);
        }
    }
}