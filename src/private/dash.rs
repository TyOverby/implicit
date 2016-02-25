use super::Point;
use itertools::PutBack;

pub struct DashSegment(pub Vec<Point>);

pub fn dashify<P, D>(points: P, dashes: D) -> Vec<DashSegment>
where P: Iterator<Item=Point>, D: Iterator<Item=f32> + Clone {
    let mut dashes = dashes.cycle();
    let mut points = PutBack::new(points);
    let mut out = vec![];

    let mut on = true;
    let mut previous = points.next();
    let mut dst = dashes.next().unwrap();

    let mut seg = vec![];
    if let Some(p) = previous {
        seg.push(p);
    }

    while let (Some(prev), Some(next)) = (previous, points.next()) {
        let prev_to_next = next - prev;
        let magnitude = prev_to_next.magnitude();
        if magnitude > dst {
            let next_break = prev + prev_to_next.normalized() * dst;

            if on {
                seg.push(next_break);
                out.push(DashSegment(seg));
                seg = vec![];
                on = false;
            } else {
                on = true;
                seg.push(next_break);
            }

            previous = Some(next_break);
            dst = dashes.next().unwrap();
            points.put_back(next);
        } else {
            if on {
                seg.push(next);
            }
            dst = dst - magnitude;
            previous = Some(next);
        }
    }

    if !seg.is_empty() {
        out.push(DashSegment(seg));
    }

    out
}
