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
    /// [1.0, 2.0, 3.0, 4.0, 5.0, 6.0] would produce the line
    ///
    /// -  ---    -----      -  ---    -----
    /// 1 2 3  4    5     6  1 2 3   4   5    6
    BasicDashed(Vec<f32>),
    /// The shape is traced with a dashed outline.
    ///
    /// The dash-segment length and gap length
    /// are stretched to repeat exactly N times.
    DashedRepeatingN(Vec<f32>, f32),
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

fn circumfrence(pts: &[Point]) -> f32 {
    if pts.len() == 0 { return 0.0; }

    let mut dist = 0.0;
    for window in pts.windows(2) {
        let p1 = window[0];
        let p2 = window[1];
        dist += p1.distance(&p2);
    }
    let first = pts[0];
    let last = pts[pts.len() - 1];
    dist += first.distance(&last);
    dist
}

fn transform(points: Vec<Vec<Point>>, mode: RenderMode) -> OutputMode {
    use super::{dashify, DashSegment};
    fn make_dash(pts: Vec<Point>, dash: &[f32]) -> DashedData {
        let dashed = dashify(pts.into_iter(), dash.iter().cloned());
        let mut lengths = Vec::with_capacity(dashed.len());
        let mut pts = vec![];
        for DashSegment(segment) in dashed {
            lengths.push(segment.len() as u32);
            pts.extend(segment);
        }
        DashedData { sizes: lengths, points: pts }
    }

    match mode {
        RenderMode::Solid => OutputMode::Solid(points),
        RenderMode::Outline => OutputMode::Outline(points),
        RenderMode::BasicDashed(dash) => {
            OutputMode::DashedLine(points.into_iter()
                                         .map(|pts| make_dash(pts, &dash[..]))
                                         .collect())
        },
        RenderMode::DashedRepeatingN(dash, n) => {
            OutputMode::DashedLine(points.into_iter().map(|pts| {
                let circ = circumfrence(&pts);
                let dash_total = dash.iter().fold(0.0, |a, b| a + b);

                let size_of_one_repeat = circ / n;
                let scale_factor = dash_total / size_of_one_repeat;
                let modified_dash = dash.iter()
                                        .map(|&l| l * scale_factor)
                                        .collect::<Vec<_>>();
                make_dash(pts, &modified_dash[..])
            }).collect())
        },
        RenderMode::DashedPerfect(dash) => {
            unimplemented!();
        }
    }
}

pub fn render<A: Implicit>(object: &A,
                           rm: RenderMode,
                           resolution: f32,
                           simplify: bool) -> OutputMode {
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

        transform(connected_lines, rm)
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
