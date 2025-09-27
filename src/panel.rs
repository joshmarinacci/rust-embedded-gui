use crate::DrawEvent;

pub fn draw_std_panel(e: &mut DrawEvent) {
    let bounds = e.view.bounds;
    e.ctx.fill_rect(&bounds, &e.theme.bg);
    e.ctx.stroke_rect(&bounds, &e.theme.fg);
}
