use super::*;
use crossbeam;
use flame;

pub struct Scene<A> {
    pub objects: Vec<A>,
    pub resolution: f32,
    pub epsilon: f32,
}

pub struct RenderedObject(pub Vec<LineType>);

impl <A: Implicit> Scene<A> {
    pub fn new(objects: Vec<A>) -> Scene<A> {
        Scene {
           objects: objects,
            resolution: 1.0,
            epsilon: 0.0001
        }
    }

    fn gather_lines<S: Implicit>(&self, sample_points: Vec<(f32, f32)>, shape: &S) -> Vec<Line> {
        let chunks = sample_points.chunks(sample_points.len() / 8 + 1);
        let chunks: Vec<Vec<_>> = chunks.map(|a| a.to_vec()).collect();
        let lines = crossbeam::scope(|scope| {
            let mut joiners = vec![];

            for chunk in chunks {
                joiners.push(scope.spawn(move || {
                    let mut local_lines = vec![];
                    for (sx, sy) in chunk {
                        match march(shape, Point {x: sx, y: sy}, self.resolution as f32) {
                            MarchResult::None => {},
                            MarchResult::One(line) => local_lines.push(line),
                            MarchResult::Two(line1, line2) => {
                                local_lines.push(line1);
                                local_lines.push(line2);
                            }
                            MarchResult::OneDebug(_) | MarchResult::Debug => { }
                        }
                    }
                    local_lines
                }));
            }

            let mut lines = vec![];
            for joiner in joiners {
                lines.append(&mut joiner.join());
            }
            lines
        });

        lines
    }

    pub fn render(&self, simplify: bool) -> Vec<RenderedObject> {
        let mut out = Vec::with_capacity(self.objects.len());
        for object in &self.objects {
            let bb = match object.bounding_box() {
                Some(bb) => bb,
                None => panic!("top level no bb"),
            };

            let sample_points = sampling_points(bb, self.resolution);
            flame::start("gather lines");
            let lines = self.gather_lines(sample_points, object);
            println!("{}", flame::end("gather lines") as f64 * 1e-9);

            let (mut connected_lines, _tree) = connect_lines(lines, self.resolution);
            if simplify {
                let mut simplified = vec![];
                for path in connected_lines {
                    match path {
                        LineType::Joined(v) =>
                            simplified.push(LineType::Joined(simplify_line(v))),
                        LineType::Unjoined(v) =>
                            simplified.push(LineType::Unjoined(simplify_line(v))),
                    }
                }
                connected_lines = simplified;
            }

            out.push(RenderedObject(connected_lines))
        }
        out
    }
}

pub fn sampling_points(bb: Rect, resolution: f32) -> Vec<(f32, f32)> {
    let start = bb.top_left;
    let end = bb.bottom_right;
    let start_x = start.x - resolution * 2.0;
    let start_y = start.y - resolution * 2.0;
    let end_x = end.x + resolution * 2.0;
    let end_y = end.y + resolution * 2.0;

    let segments_x = (end_x - start_x) / resolution;
    let segments_y = (end_y - start_y) / resolution;
    let num_points = segments_x * segments_y;

    let mut x = start_x;
    let mut y = start_y;
    let mut out = Vec::with_capacity(num_points.ceil() as usize);

    while y < end_y {
        while x < end_x {
            out.push((x, y));
            x += resolution;
        }
        x = start_x;
        y += resolution;
    }
    out
}
