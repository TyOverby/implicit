use ::{OutputMode, RenderMode, SyncImplicit, Implicit, render, OutputDevice};
use ::geom::{Point, Rect};

pub struct Scene {
    shapes: Vec<((f32, f32), OutputMode)>,
    pub resolution: f32,
    total_bounding_box: Rect,
}

impl  Scene {
    pub fn new() -> Scene {
        Scene {
            shapes: Vec::new(),
            total_bounding_box: Rect::null_at(&Point{x: 0.0, y: 0.0}),
            resolution: 2.0,
        }
    }

    pub fn add_shape<I: SyncImplicit>(&mut self, shape: &I, offset: (f32, f32), rendermode: RenderMode) -> usize {
        let i = self.shapes.len();
        self.total_bounding_box = self.total_bounding_box.union_with(&shape.bounding_box().unwrap());
        self.shapes.push((offset, render(shape, &rendermode, self.resolution, true)));
        i
    }

    pub fn add_again(&mut self, again: usize, offset: (f32, f32)) {
        let mut old = self.shapes[again].clone();
        old.0 = offset;
        self.shapes.push(old);
    }

    pub fn render_all<O: OutputDevice>(&self, out: &mut O) {
        for &((ox, oy), ref rendered) in &self.shapes {
            match rendered {
                &OutputMode::Solid(_) => unimplemented!(),
                &OutputMode::Outline(ref lines) => {
                    for line in lines {
                        out.start_line();
                        let Point{x: sx, y: sy} = line[0];
                        for &Point{x, y} in line {
                            out.add_point(x + ox, y + oy);
                        }
                        out.add_point(sx + ox, sy + oy);
                        out.end_line();
                    }
                },
                &OutputMode::DashedLine(ref dashed) => {
                    for dashed_line in dashed {
                        for segment in dashed_line.segments() {
                            out.start_line();
                            for &Point{x, y} in segment {
                                out.add_point(x + ox, y + oy);
                            }
                            out.end_line();
                        }
                    }
                }
            }
        }
        out.set_size(self.total_bounding_box.width(), self.total_bounding_box.height());
    }
}
