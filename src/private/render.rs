use super::{simplify_line, connect_lines, Point, MarchResult, march, Rect, Line};

use ::Implicit;
use crossbeam;
use flame;

pub enum RenderMode {
    /// The shape is filled in and completely solid.
    Solid,
    /// The shape is traced with an outline.
    Outline,
    /// The shape is traced with a dashed outline.
    ///
    /// The dash-segment length and gap length are
    /// pulled from the vector in an alternating
    /// pattern.
    ///
    /// [1.0, 2.0, 3.0, 4.0, 5.0] would produce the line
    /// -  ---    -----
    /// 1 2 3  4    5
    BasicDashed(Vec<f32>),
    /// The shape is traced with a dashed outline.
    ///
    /// The dash-segment length and gap length
    /// are stretched to repeat exactly N times.
    DashedRepeatingN(Vec<f32>, u32),
    /// The shape is traced with a dashed outline that
    /// wraps around to end exactly where it began.
    ///
    /// The dash-segment length and gap length are
    /// stretched the smallest amount to make the
    /// ends meet.
    DashedPerfect(Vec<f32>)
}

pub enum OutputMode {
    Solid(Vec<Vec<Point>>),
    Outline(Vec<Vec<Point>>),
    DashedLine(Vec<DashedData>)
}

pub struct DashedData {
    sizes: Vec<u32>,
    points: Vec<Point>
}

pub struct SegmentIter<'a> {
    data: &'a DashedData,
    last_segment_idx: usize,
    last_points_pos: usize,
}

impl <'a> Iterator for SegmentIter<'a> {
    type Item = &'a[Point];
    fn next(&mut self) -> Option<&'a [Point]> {
        if self.last_segment_idx >= self.data.sizes.len() {
            return None
        } 
        let size_of_segment = self.data.sizes[self.last_segment_idx + 1];
        let slice = &self.data.points[
            self.last_points_pos ..
            self.last_points_pos + size_of_segment as usize];

        self.last_segment_idx += 1;
        self.last_points_pos += size_of_segment as usize;
        Some(slice)
    }
}

impl DashedData {
    fn segments(&self) -> SegmentIter {
        SegmentIter {
            data: self,
            last_segment_idx: 0,
            last_points_pos: 0,
        }
    }
}

pub fn render<A: Implicit>(object: &A, resolution: f32, simplify: bool) -> Vec<Vec<Point>> {
        let bb = match object.bounding_box() {
            Some(bb) => bb,
            None => panic!("top level no bb"),
        };

        let sample_points = sampling_points(bb, resolution);
        flame::start("gather lines");
        let lines = gather_lines(resolution, sample_points, object);
        flame::end("gather lines");

        let (mut connected_lines, _tree) = connect_lines(lines, resolution);
        if simplify {
            connected_lines = connected_lines.into_iter().map(simplify_line).collect();
        }

        connected_lines
}

fn gather_lines<S: Implicit>(resolution: f32, sample_points: Vec<(f32, f32)>, shape: &S) -> Vec<Line> {
    let chunks = sample_points.chunks(sample_points.len() / 8 + 1);
    let chunks: Vec<Vec<_>> = chunks.map(|a| a.to_vec()).collect();
    let lines = crossbeam::scope(|scope| {
        let mut joiners = vec![];

        for chunk in chunks {
            joiners.push(scope.spawn(move || {
                let mut local_lines = vec![];
                for (sx, sy) in chunk {
                    match march(shape, Point {x: sx, y: sy}, resolution) {
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
