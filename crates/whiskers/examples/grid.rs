use whiskers::{grid::Grid, prelude::*};

#[derive(Sketch)]
struct GridSketch {
    #[param(slider, min = 20.0, max = 400.0)]
    width: f64,
    #[param(slider, min = 20.0, max = 400.0)]
    height: f64,
    #[param(slider, min = 2, max = 20)]
    columns: usize,
    #[param(slider, min = 2, max = 20)]
    rows: usize,
    #[param(slider, min = 0.0, max = 200.0)]
    gutter_width: f64,
    #[param(slider, min = 0.0, max = 200.0)]
    gutter_height: f64,
    is_canvas_sizing: bool,
    marked_cell_col: usize,
    marked_cell_row: usize,
}

impl Default for GridSketch {
    fn default() -> Self {
        Self {
            width: 100.0,
            height: 100.0,
            columns: 5,
            rows: 5,
            gutter_width: 20.0,
            gutter_height: 20.0,
            is_canvas_sizing: false,
            marked_cell_col: 0,
            marked_cell_row: 0,
        }
    }
}

impl App for GridSketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        sketch.stroke_width(5.0);

        let mut grid = if self.is_canvas_sizing {
            Grid::from_total_size([sketch.width(), sketch.height()])
        } else {
            Grid::from_cell_size([self.width, self.height])
        };

        grid.columns(self.columns)
            .rows(self.rows)
            .spacing([self.gutter_width, self.gutter_height])
            .build(sketch, |sketch, cell| {
                let c = cell.clone();
                sketch.color(Color::RED);
                sketch.add_path(c);
            });

        if let Some(marked_cell) = grid.at(self.marked_cell_col, self.marked_cell_row) {
            sketch.color(Color::GREEN);
            sketch.add_path(marked_cell.clone());
        }

        Ok(())
    }
}

fn main() -> Result {
    Runner::new(GridSketch::default())
        .with_page_size_options(PageSize::A5H)
        .run()
}