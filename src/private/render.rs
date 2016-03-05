use super::{simplify_line, connect_lines, Point, MarchResult, march, Rect, Line};

use ::Implicit;
use std::cmp::{PartialOrd, Ordering};
use itertools::Itertools;
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

#[derive(Clone)]
pub enum OutputMode {
    Solid(Vec<Vec<Point>>),
    Outline(Vec<Vec<Point>>),
    DashedLine(Vec<DashedData>)
}
pub struct SegmentIter<'a> {
    data: &'a DashedData,
    last_segment_idx: usize,
    last_points_pos: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct SampleDist {
    pub x_bump: f32,
    pub y_bump: f32,
}

#[derive(Clone)]
pub struct DashedData {
    sizes: Vec<u32>,
    points: Vec<Point>
}

impl SampleDist {
    fn modify_bb(&self, bb: &mut Rect) {
        let top_left = {
            let Point{ x, y } = bb.top_left();
            let (x, y) = self.floor(x, y);
            Point{x: x, y: y}
        };
        let bottom_right = {
            let Point { x, y } = bb.bottom_right();
            let (x, y) = self.floor(x, y);
            Point{x: x, y: y}
        };

        *bb = Rect::from_points(&top_left, &bottom_right);
    }
    fn floor(&self, x: f32, y: f32) -> (f32, f32){
        let x = x - (x % self.x_bump);
        let y = y - (y % self.y_bump);
        (x, y)
    }
    fn bump_x(&self, x: f32) -> f32 {
        x + self.x_bump
    }
    fn bump_y(&self, x: f32) -> f32 {
        x + self.x_bump
    }
    fn max_bump(&self) -> f32 {
        self.x_bump.max(self.y_bump)
    }
}

impl <'a> Iterator for SegmentIter<'a> {
    type Item = &'a[Point];
    fn next(&mut self) -> Option<&'a [Point]> {
        if self.last_segment_idx >= self.data.sizes.len() {
            return None
        } 
        let size_of_segment = self.data.sizes[self.last_segment_idx];
        let slice = &self.data.points[
            self.last_points_pos ..
            self.last_points_pos + size_of_segment as usize];

        self.last_segment_idx += 1;
        self.last_points_pos += size_of_segment as usize;
        Some(slice)
    }
}

impl DashedData {
    pub fn segments(&self) -> SegmentIter {
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

fn transform(points: Vec<Vec<Point>>, mode: &RenderMode) -> OutputMode {
    use super::{dashify, DashSegment};
    fn make_dash(mut pts: Vec<Point>, dash: &[f32]) -> DashedData {
        let (min, _) = pts.iter().cloned().enumerate().fold1(|(pi, pp), (ni, np)| {
            const REALLY_FAR_AWAY: Point = Point{x: -100000.0, y: -1000000.0};
            if pp.distance_2(&REALLY_FAR_AWAY) < np.distance_2(&REALLY_FAR_AWAY) {
                (pi, pp)
            } else {
                (ni, np)
            }
        }).unwrap();

        rotate_and_correct(&mut pts, min + 1);
        let first = *pts.first().unwrap();
        pts.push(first);

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
        &RenderMode::Solid => OutputMode::Solid(points),
        &RenderMode::Outline => OutputMode::Outline(points),
        &RenderMode::BasicDashed(ref dash) => {
            OutputMode::DashedLine(points.into_iter()
                                         .map(|pts| make_dash(pts, &dash[..]))
                                         .collect())
        },
        &RenderMode::DashedRepeatingN(ref dash, n) => {
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
        &RenderMode::DashedPerfect(ref dash) => {
            OutputMode::DashedLine(points.into_iter().map(|pts| {
                let circ: f32 = circumfrence(&pts);
                let dash_total = dash.iter().fold(0.0, |a, b| a + b);

                // If (circ / dash_total) is a whole number, then it's a perfect loop
                // so scale factor is 1. It probably isn't, so lets round.
                let dash_ratio = circ / dash_total;
                let r = (dash_ratio).round();
                let s = (circ / r) / dash_total;

                let modified_dash = dash.iter()
                                        .map(|&l| l * s)
                                        .collect::<Vec<_>>();
                make_dash(pts, &modified_dash[..])
            }).collect())
        }
    }
}

pub fn render<A>(object: A, rm: &RenderMode, resolution: f32, simplify: bool) -> OutputMode
where A: Implicit + Sync {

    const FACTOR: f32 = 1.0;
//    let object = Implicit::scale(object, FACTOR, FACTOR);
//    let resolution = resolution * FACTOR;
    let sample_points = sampling_points(&object, resolution);
    flame::start("gather lines");
    let lines = gather_lines(resolution, sample_points, &object);
    flame::end("gather lines");

    let (mut connected_lines, _tree) = connect_lines(lines, resolution);
    if simplify {
        connected_lines = connected_lines.into_iter().map(simplify_line).collect();
    }

    for group in &mut connected_lines {
        for &mut Point { ref mut x, ref mut y } in group.iter_mut() {
            *x /= FACTOR;
            *y /= FACTOR;
        }
    }

    transform(connected_lines, rm)
}

fn gather_lines<S: Implicit + Sync>(resolution: f32, sample_points: Vec<(f32, f32)>, shape: &S) -> Vec<Line> {
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

pub fn sampling_points<S: Implicit>(shape: &S, resolution: f32) -> Vec<(f32, f32)> {
    let bb = shape.bounding_box().unwrap();
    let b_dim = bb.width().max(bb.height());
    let expand = b_dim * 0.10;
    let bb = bb.expand(expand, expand, expand, expand);

    assert!(!bb.is_null(), "shape is null");
    let sample_dist = SampleDist {
        x_bump: resolution,
        y_bump: resolution,
    };
    let mut out = vec![];

    fn subdivide<S: Implicit>(shape: &S, bb: Rect, sample_dist: SampleDist, out: &mut Vec<Point>) {
        let radius = bb.width().max(bb.height());
        let sample = shape.sample(bb.midpoint()).abs();

        if sample > radius {
            return
        }
        if  radius < sample_dist.max_bump() * 10.0 || radius < 1.0 {
            sample_from_box(bb, sample_dist, out);
            return;
        }

        let q = bb.split_quad();
        subdivide(shape, q[0], sample_dist, out);
        subdivide(shape, q[1], sample_dist, out);
        subdivide(shape, q[2], sample_dist, out);
        subdivide(shape, q[3], sample_dist, out);
    }

    if shape.follows_rules() {
        subdivide(shape, bb, sample_dist, &mut out);
    } else {
        sample_from_box(bb, sample_dist, &mut out);
    }

    out.sort_by(|a, b| {
        match a.x.partial_cmp(&b.x) {
            Some(a) => a,
            None => Ordering::Equal
        }
    });
    remove_similar(&mut out);

    out.sort_by(|a, b| {
        match a.y.partial_cmp(&b.y) {
            Some(a) => a,
            None => Ordering::Equal
        }
    });
    remove_similar(&mut out);

    // TODO: make this function return points
    out.into_iter().map(|p| p.into_tuple()).collect()
}

fn remove_similar(out: &mut Vec<Point>) {
    let mut last = None;
    let mut to_remove: Vec<usize> = vec![];

    // Build up a list of indices to remove.
    for (i, &pt) in out.iter().enumerate() {
        if last.is_none() {
            last = Some(pt);
            continue;
        }
        let last_u = last.unwrap();
        if pt.close_to(&last_u, 0.01) {
            to_remove.push(i);
        } 
        last = Some(pt);
    }

    // Reverse the list so that we can "pop" from the front
    to_remove.reverse();

    // Drop all the removed indicies
    let mut i = 0;
    out.retain(|_| {
        if to_remove.is_empty() {
            return true;
        }

        let &last_idx = to_remove.last().unwrap();
        let result = if last_idx == i {
            to_remove.pop();
            false
        } else {
            true
        };

        i += 1;
        result
    });
}

fn sample_from_box(mut bb: Rect, sample_dist: SampleDist, out: &mut Vec<Point>) {
    sample_dist.modify_bb(&mut bb);
    let Point{mut x, mut y} = bb.top_left();
    let x_orig = x;

    loop {
        let p = Point{x: x, y: y};
        if bb.contains(&p) {
            out.push(p);
            x = sample_dist.bump_x(x);
        } else {
            x = x_orig;
            y = sample_dist.bump_y(y);
            if !bb.contains(&Point{x: x, y: y}) { break; }
        }
    }
}

fn rotate_and_correct(slice: &mut [Point], point: usize) {
    fn is_clockwise(points: &[Point]) -> bool {
        let mut total = 0.0;
        for slice in points.windows(2) {
            let a = slice[0];
            let b = slice[1];
            total += (b.x - a.x) * (b.y + a.y);
        }
        total > 0.0
    }

    {
        let (a, b) = slice.split_at_mut(point);
        a.reverse();
        b.reverse();
    }

    if !is_clockwise(slice) {
        slice.reverse();
    }
}

