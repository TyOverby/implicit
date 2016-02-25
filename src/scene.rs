use ::{OutputMode, RenderMode, SyncBox, SyncImplicit, Implicit, render, OutputDevice};
use ::geom::Point;

pub struct Scene {
    shapes: Vec<(SyncBox, RenderMode)>,
    pub resolution: f32
}

impl  Scene {
    pub fn new() -> Scene {
        Scene {
            shapes: Vec::new(),
            resolution: 2.0,
        }
    }

    pub fn add_shape<I: SyncImplicit + 'static>(&mut self, shape: I, rendermode: RenderMode) {
        self.shapes.push((shape.boxed(), rendermode));
    }

    pub fn render_all<O: OutputDevice>(&self, out: &mut O) {
        for &(ref shape, ref mode) in self.shapes.iter() {
            match render(shape, mode, self.resolution, true) {
                OutputMode::Solid(_) => unimplemented!(),
                OutputMode::Outline(lines) => {
                    for line in lines {
                        out.start_line();
                        let Point{x: sx, y: sy} = line[0];
                        for Point{x, y} in line {
                            out.add_point(x, y);
                        }
                        out.add_point(sx, sy);
                        out.end_line();
                    }
                },
                OutputMode::DashedLine(dashed) => {
                    for dashed_line in dashed {
                        for segment in dashed_line.segments() {
                            out.start_line();
                            for &Point{x, y} in segment {
                                out.add_point(x, y);
                            }
                            out.end_line();
                        }
                    }
                }
            }
        }
    }
}
