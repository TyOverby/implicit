use ::{OutputMode, RenderMode, SyncImplicit, Implicit, render, OutputDevice};
use ::geom::{Point, Rect, Matrix};

pub struct Scene {
    shapes: Vec<(Rect, Matrix, OutputMode)>,
    pub resolution: f32,
    total_bounding_box: Rect,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            shapes: Vec::new(),
            total_bounding_box: Rect::null_at(&Point{x: 0.0, y: 0.0}),
            resolution: 2.0,
        }
    }

    pub fn add_shape<I: SyncImplicit>(&mut self, shape: &I, rendermode: RenderMode, matrix: Matrix) -> usize {
        let i = self.shapes.len();
        let bb = shape.bounding_box().unwrap();
        let new_bb = transform_bounding_box(bb, matrix);

        self.total_bounding_box = self.total_bounding_box.union_with(&new_bb);
        self.shapes.push((shape.bounding_box().unwrap(), matrix, render(shape, &rendermode, self.resolution, true)));
        i
    }

    pub fn add_again(&mut self, again: usize, matrix: Matrix) {
        let mut old = self.shapes[again].clone();
        old.1 = matrix;
        old.0 = transform_bounding_box(old.0, matrix);

        self.total_bounding_box = self.total_bounding_box.union_with(&old.0);
        self.shapes.push(old);
    }

    pub fn render_all<O: OutputDevice>(&self, out: &mut O) {
        out.set_size(self.total_bounding_box.width(), self.total_bounding_box.height());
        for &(_, matrix, ref rendered) in &self.shapes {
            match rendered {
                &OutputMode::Solid(_) => unimplemented!(),
                &OutputMode::Outline(ref lines) => {
                    for line in lines {
                        out.start_line();
                        let start = line[0];
                        for p in line {
                            out.add_point(matrix.transform_point(p));
                        }
                        out.add_point(matrix.transform_point(&start));
                        out.end_line();
                    }
                },
                &OutputMode::DashedLine(ref dashed) => {
                    for dashed_line in dashed {
                        for segment in dashed_line.segments() {
                            out.start_line();
                            for p in segment {
                                out.add_point(matrix.transform_point(p));
                            }
                            out.end_line();
                        }
                    }
                }
            }
        }
    }
}

pub fn transform_bounding_box(bb: Rect, matrix: Matrix) -> Rect {
    let a = matrix.transform_point(&bb.top_left());
    let b = matrix.transform_point(&bb.top_right());
    let c = matrix.transform_point(&bb.bottom_left());
    let d = matrix.transform_point(&bb.bottom_right());
    let mut new_bb = Rect::null_at(&a);
    new_bb.expand_to_include(&b);
    new_bb.expand_to_include(&c);
    new_bb.expand_to_include(&d);
    new_bb
}
