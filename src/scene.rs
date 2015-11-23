use super::*;
use std::vec;

pub struct Scene<'a> {
    pub objects: Vec<GenericShape<'a>>,
    pub resolution: f32,
    pub epsilon: f32,
}

pub struct RenderedObject(pub Vec<LineType>);

impl <'a> Scene<'a> {
    pub fn new(objects: Vec<GenericShape<'a>>) -> Scene {
        Scene {
            objects: objects,
            resolution: 1.0,
            epsilon: 0.0001
        }
    }

    pub fn render(&self, simplify: bool) -> Vec<RenderedObject> {
        let mut out = Vec::with_capacity(self.objects.len());
        for object in &self.objects {
            let bb = match object.bounding_box() {
                Some(bb) => bb,
                None => panic!("top level no bb"),
            };

            let sample_points = sampling_points(bb, self.resolution);

            let mut lines = vec![];
            for (sx, sy) in sample_points {
                match march(object, Point {x: sx, y: sy}, self.resolution as f32) {
                    MarchResult::None => {},
                    MarchResult::One(line) => lines.push(line),
                    MarchResult::Two(line1, line2) => {
                        lines.push(line1);
                        lines.push(line2);
                    }
                    MarchResult::OneDebug(_) | MarchResult::Debug => { }
                }
            }

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

pub fn sampling_points(bb: Rect, resolution: f32) -> vec::IntoIter<(f32, f32)> {
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
    out.into_iter()
}
