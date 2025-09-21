use crate::geom::Bounds;
use crate::view::View;
use crate::{DrawEvent, HAlign, LayoutEvent, VAlign};
use alloc::boxed::Box;
use alloc::string::String;
use hashbrown::HashMap;

pub struct FormLayoutState {
    pub constraints: HashMap<String, LayoutConstraint>,
    row_count: usize,
    col_count: usize,
    col_width: usize,
    row_height: usize,
}

impl FormLayoutState {
    pub fn new_row_column(
        row_count: usize,
        row_height: usize,
        col_count: usize,
        col_width: usize,
    ) -> FormLayoutState {
        FormLayoutState {
            constraints: HashMap::new(),
            col_count,
            row_count,
            col_width,
            row_height,
        }
    }
}

impl FormLayoutState {
    pub fn place_at_row_column(
        &mut self,
        name: &str,
        row: usize,
        col: usize,
    ) -> Option<LayoutConstraint> {
        self.constraints
            .insert(name.into(), LayoutConstraint::at_row_column(row, col))
    }
}

pub struct LayoutConstraint {
    col: usize,
    row: usize,
    col_span: usize,
    row_span: usize,
    h_align: HAlign,
    v_align: VAlign,
}

impl LayoutConstraint {
    pub fn at_row_column(row: usize, col: usize) -> LayoutConstraint {
        LayoutConstraint {
            col,
            row,
            col_span: 1,
            row_span: 1,
            h_align: HAlign::Center,
            v_align: VAlign::Center,
        }
    }
}

pub fn make_form(name: &str) -> View {
    View {
        name: name.into(),
        title: name.into(),
        bounds: Bounds::new(0, 0, 100, 100),
        input: None,
        state: Some(Box::new(FormLayoutState {
            constraints: HashMap::new(),
            col_count: 2,
            row_count: 2,
            col_width: 100,
            row_height: 30,
        })),
        layout: Some(layout_form),
        draw: Some(common_draw_panel),
        visible: true,
    }
}

fn common_draw_panel(evt: &mut DrawEvent) {
    evt.ctx.fill_rect(&evt.view.bounds, &evt.theme.bg);
    evt.ctx.stroke_rect(&evt.view.bounds, &evt.theme.fg);
}

fn layout_form(evt: &mut LayoutEvent) {
    if let Some(view) = evt.scene.get_view(evt.target) {
        let parent_bounds = view.bounds;
        let kids = evt.scene.get_children(evt.target);
        for kid in kids {
            if let Some(state) = evt.scene.get_view_state::<FormLayoutState>(evt.target) {
                let bounds = if let Some(cons) = &state.constraints.get(&kid) {
                    let x = (cons.col * state.col_width) as i32;
                    let y = (cons.row * state.row_height) as i32;
                    let w = state.col_width as i32;
                    let h = state.row_height as i32;
                    Bounds::new(x, y, w, h)
                } else {
                    Bounds::new(0, 0, 0, 0)
                };
                if let Some(view) = evt.scene.get_view_mut(&kid) {
                    view.bounds = bounds;
                }
            }
        }
    }
}

mod tests {
    use crate::comps::make_label;
    use crate::form::{FormLayoutState, make_form};
    use crate::geom::Bounds;
    use crate::scene::{Scene, draw_scene, layout_scene};
    use crate::{MockDrawingContext, Theme};
    use alloc::boxed::Box;
    use alloc::string::String;
    use embedded_graphics::mock_display::MockDisplay;
    use embedded_graphics::mono_font::ascii::{FONT_7X13, FONT_7X13_BOLD};
    use embedded_graphics::mono_font::MonoFont;
    use embedded_graphics::pixelcolor::{Rgb565, RgbColor, WebColors};

    #[test]
    fn test_form_layout() {
        let theme = MockDrawingContext::make_mock_theme();

        let mut form = make_form("form");
        form.bounds.x = 40;
        form.bounds.y = 40;
        form.bounds.w = 200;
        form.bounds.h = 200;
        let mut form_layout = FormLayoutState::new_row_column(2, 30, 2, 100);

        let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

        let label1 = make_label("label1", "Label 1");
        form_layout.place_at_row_column(&label1.name, 0, 0);
        scene.add_view_to_parent(label1, &form.name);

        let label2 = make_label("label2", "Label 2");
        form_layout.place_at_row_column(&label2.name, 0, 1);
        scene.add_view_to_parent(label2, &form.name);

        let label3 = make_label("label3", "Label 3");
        form_layout.place_at_row_column(&label3.name, 1, 0);
        scene.add_view_to_parent(label3, &form.name);

        form.state = Some(Box::new(form_layout));
        scene.add_view_to_root(form);

        layout_scene(&mut scene, &theme);

        {
            let label1 = scene.get_view("label1").unwrap();
            assert_eq!(label1.name, "label1");
            assert_eq!(label1.bounds, Bounds::new(40, 40, 63, 25));

            let label2 = scene.get_view("label2").unwrap();
            assert_eq!(label2.name, "label2");
            assert_eq!(label2.bounds, Bounds::new(140, 40, 63, 25));

            let label3 = scene.get_view("label3").unwrap();
            assert_eq!(label3.name, "label3");
            assert_eq!(label3.bounds, Bounds::new(40, 70, 63, 25));
        }

        let mut ctx = MockDrawingContext::new(&scene);

        assert_eq!(scene.dirty, true);
        draw_scene(&mut scene, &mut ctx, &theme);
        assert_eq!(scene.dirty, false);
    }
}
