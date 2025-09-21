use crate::geom::Bounds;
use crate::view::View;
use crate::{DrawEvent, HAlign, LayoutEvent, VAlign};
use alloc::boxed::Box;
use alloc::string::String;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use hashbrown::HashMap;

pub struct GridLayoutState {
    pub constraints: HashMap<String, LayoutConstraint>,
    row_count: usize,
    col_count: usize,
    col_width: usize,
    row_height: usize,
    pub debug:bool
}

impl GridLayoutState {
    pub fn new_row_column(
        row_count: usize,
        row_height: usize,
        col_count: usize,
        col_width: usize,
    ) -> GridLayoutState {
        GridLayoutState {
            constraints: HashMap::new(),
            col_count,
            row_count,
            col_width,
            row_height,
            debug: false,
        }
    }
}

impl GridLayoutState {
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
    pub col: usize,
    pub row: usize,
    pub col_span: usize,
    pub row_span: usize,
    pub h_align: HAlign,
    pub v_align: VAlign,
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

pub fn make_grid_panel(name: &str) -> View {
    View {
        name: name.into(),
        title: name.into(),
        bounds: Bounds::new(0, 0, 100, 100),
        input: None,
        state: Some(Box::new(GridLayoutState {
            constraints: HashMap::new(),
            col_count: 2,
            row_count: 2,
            col_width: 100,
            row_height: 30,
            debug: false,
        })),
        layout: Some(layout_grid),
        draw: Some(draw_grid),
        visible: true,
    }
}

fn draw_grid(evt: &mut DrawEvent) {
    evt.ctx.fill_rect(&evt.view.bounds, &evt.theme.bg);
    evt.ctx.stroke_rect(&evt.view.bounds, &evt.theme.fg);

    let bounds = evt.view.bounds;
    if let Some(state) = evt.view.get_state::<GridLayoutState>() {
        if state.debug {
            for i in 0..state.col_count {
                for j in 0 .. state.row_count {
                    let rect = Bounds::new(
                        (i * state.col_width) as i32 + bounds.x,
                        (j * state.row_height) as i32 + bounds.y,
                           state.col_width as i32,
                           state.row_height as i32
                    );
                    evt.ctx.stroke_rect(&rect, &Rgb565::RED);
                }
            }
        }
    }
}

fn layout_grid(evt: &mut LayoutEvent) {
    if let Some(view) = evt.scene.get_view(evt.target) {
        let parent_bounds = view.bounds;
        let kids = evt.scene.get_children(evt.target);
        for kid in kids {
            if let Some(state) = evt.scene.get_view_state::<GridLayoutState>(evt.target) {
                let cell_bounds = if let Some(cons) = &state.constraints.get(&kid) {
                    let x = (cons.col * state.col_width) as i32;
                    let y = (cons.row * state.row_height) as i32;
                    let w = state.col_width as i32 * cons.col_span as i32;
                    let h = state.row_height as i32 * cons.row_span as i32;
                    Bounds::new(x, y, w, h)
                } else {
                    Bounds::new(0, 0, 0, 0)
                };
                if let Some(view) = evt.scene.get_view_mut(&kid) {
                    center_within(cell_bounds, &mut view.bounds);
                    // view.bounds = cell_bounds;
                }
            }
        }
    }
}

fn center_within(cell: Bounds, view: &mut Bounds) {
    view.x = (cell.w - view.w)/2 + cell.x;
    view.y = (cell.h - view.h)/2 + cell.y;
}

mod tests {
    use crate::comps::{make_button, make_label};
    use crate::grid::{GridLayoutState, make_grid_panel, LayoutConstraint};
    use crate::geom::Bounds;
    use crate::scene::{Scene, draw_scene, layout_scene};
    use crate::{HAlign, MockDrawingContext, Theme, VAlign};
    use alloc::boxed::Box;
    use alloc::string::String;
    use embedded_graphics::mock_display::MockDisplay;
    use embedded_graphics::mono_font::MonoFont;
    use embedded_graphics::mono_font::ascii::{FONT_7X13, FONT_7X13_BOLD};
    use embedded_graphics::pixelcolor::{Rgb565, RgbColor, WebColors};

    #[test]
    fn test_grid_layout() {
        let theme = MockDrawingContext::make_mock_theme();

        let mut grid = make_grid_panel("grid");
        grid.bounds.x = 40;
        grid.bounds.y = 40;
        grid.bounds.w = 200;
        grid.bounds.h = 200;
        let mut grid_layout = GridLayoutState::new_row_column(2, 30, 2, 100);

        let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

        let label1 = make_label("label1", "Label 1");
        grid_layout.place_at_row_column(&label1.name, 0, 0);
        scene.add_view_to_parent(label1, &grid.name);

        let label2 = make_label("label2", "Label 2");
        grid_layout.place_at_row_column(&label2.name, 0, 1);
        scene.add_view_to_parent(label2, &grid.name);

        let label3 = make_label("label3", "Label 3");
        grid_layout.place_at_row_column(&label3.name, 1, 0);
        scene.add_view_to_parent(label3, &grid.name);

        grid.state = Some(Box::new(grid_layout));
        scene.add_view_to_root(grid);

        layout_scene(&mut scene, &theme);

        {
            let label1 = scene.get_view("label1").unwrap();
            assert_eq!(label1.name, "label1");
            assert_eq!(label1.bounds, Bounds::new(0, 0, 63, 25));

            let label2 = scene.get_view("label2").unwrap();
            assert_eq!(label2.name, "label2");
            assert_eq!(label2.bounds, Bounds::new(100, 0, 63, 25));

            let label3 = scene.get_view("label3").unwrap();
            assert_eq!(label3.name, "label3");
            assert_eq!(label3.bounds, Bounds::new(0, 70, 63, 25));
        }

        let mut ctx = MockDrawingContext::new(&scene);

        assert_eq!(scene.dirty, true);
        draw_scene(&mut scene, &mut ctx, &theme);
        assert_eq!(scene.dirty, false);
    }

    #[test]
    fn col_span() {
        let theme = MockDrawingContext::make_mock_theme();
        let mut grid = make_grid_panel("grid").position_at(0,0).with_size(200,200);
        let mut layout = GridLayoutState::new_row_column(2, 30, 2, 100);
        layout.debug = true;
        let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

        let button = make_button("b1","b1");
        layout.constraints.insert((&button.name).into(),LayoutConstraint{
            col:0,
            row:0,
            col_span: 2,
            row_span: 1,
            h_align: HAlign::Center,
            v_align: VAlign::Center,
        });

        grid.state = Some(Box::new(layout));
        scene.add_view_to_parent(button,&grid.name);
        scene.add_view_to_root(grid);
        layout_scene(&mut scene, &theme);

        if let Some(view) = scene.get_view("b1") {
            assert_eq!(view.bounds, Bounds::new(0,0,200,30));
        }
    }
}
