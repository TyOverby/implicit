use super::{
    simplify_line,
    connect_lines,
    Point,
    MarchResult,
    march,
    Line,
    A, B, C, D,
    sampling_points,
};

use ::Implicit;
use itertools::Itertools;
use flame;

#[derive(Clone)]
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

#[derive(Clone)]
pub struct DashedData {
    sizes: Vec<u32>,
    points: Vec<Point>
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
        // Make all of the points go clockwise
        correct_spin(&mut pts);

        // Find the smallest point in this line segment
        let (min, closest) = pts.iter().cloned().enumerate().fold1(|(pi, pp), (ni, np)| {
            const REALLY_FAR_AWAY: Point = Point{x: -100000.0, y: -1000000.0};
            if pp.distance_2(&REALLY_FAR_AWAY) < np.distance_2(&REALLY_FAR_AWAY) {
                (pi, pp)
            } else {
                (ni, np)
            }
        }).unwrap();

        // Make the "smallest point" the first in the series.
        rotate(&mut pts, min);

        // Copy the first point to the end in order to complete the chain
        let first = *pts.first().unwrap();
        assert_eq!(first, closest);
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
    flame::start("collect sampling points");
    let sample_points = sampling_points(&object, resolution);
    flame::end("collect sampling points");

    flame::start("gather lines");
    let lines = gather_lines(resolution, sample_points, &object);
    flame::end("gather lines");

    flame::start("connect lines");
    let (mut connected_lines, _tree) = connect_lines(lines, resolution);
    if simplify {
        connected_lines = connected_lines.into_iter().map(simplify_line).collect();
    }
    flame::end("connect lines");

    flame::start("transform lines");
    let r = transform(connected_lines, rm);
    flame::end("transform lines");
    r
}

fn gather_lines<S: Implicit + Sync>(resolution: f32, sample_points: Vec<(f32, f32)>, shape: &S) -> Vec<Line> {
    fn divide<S: Implicit + Sync>(shape: &S, chunks: &[&[(f32, f32)]], resolution: f32) -> Vec<Line> {
        if chunks.len() == 0 {
            return vec![];
        }
        if chunks.len() == 1{
            return sample_these(shape, chunks[0], resolution);
        }

        let (mut cur, mut rest) = ::rayon::join(
            || sample_these(shape, chunks[0], resolution),
            || divide(shape, &chunks[1..], resolution));
        rest.append(&mut cur);

        rest
    }

    let chunks: Vec<_> = sample_points.chunks(sample_points.len() / ::num_cpus::get()).collect();
    divide(shape, &chunks, resolution)
}

fn sample_these<S: Implicit>(shape: &S, chunk: &[(f32, f32)], resolution: f32) -> Vec<Line> {
    let mut local_lines = vec![];

    // Previously sampled points
    let mut p_right_top: Option<(Point, f32)> = None;
    let mut p_right_bot: Option<(Point, f32)> = None;

    for &(sx, sy) in chunk {
        let p = Point{x: sx, y: sy};

        let sa = A * resolution + p;
        let sb = B * resolution + p;
        let sc = C * resolution + p;
        let sd = D * resolution + p;

        ::flame::start("sampling");
        let sra = match p_right_top {
            Some((pp, pv)) if pp.close_to(&sa, resolution) => {
                pv
            }
            _ => {
                shape.sample(sa)
            }
        };

        let srd = match p_right_bot {
            Some((pp, pv)) if pp.close_to(&sd, resolution) => {
                pv
            }
            _ => {
                shape.sample(sd)
            }
        };

        let srb = shape.sample(sb);
        let src = shape.sample(sc);
        ::flame::end("sampling");

        p_right_top = Some((sb, srb));
        p_right_bot = Some((sc, src));

        match march(sra, srb, src, srd, shape, p, resolution) {
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
}

fn correct_spin(points: &mut [Point]) {
    fn is_clockwise(points: &[Point]) -> bool {
        let mut total = 0.0;
        for slice in points.windows(2) {
            let a = slice[0];
            let b = slice[1];
            total += (b.x - a.x) * (b.y + a.y);
        }
        total > 0.0
    }

    if !is_clockwise(points) {
        points.reverse();
    }
}

fn rotate<T>(slice: &mut [T], at: usize) {
    {
        let (a, b) = slice.split_at_mut(at);
        a.reverse();
        b.reverse();
    }

    slice.reverse();
}

#[test]
fn rotation_is_correct() {
    let mut slice = [0, 1, 2, 3, 4, 5];
    rotate(&mut slice, 3);
    assert_eq!(slice[0], 3);
}

