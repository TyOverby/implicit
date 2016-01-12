use std::cmp::{PartialOrd, Ordering};

use super::*;

const EPSILON: f32 = 0.001;

#[derive(Debug)]
pub enum LineType {
    Joined(Vec<Point>),
    Unjoined(Vec<Point>)
}

impl LineType {
    pub fn points(self) -> Vec<Point> {
        match self {
            LineType::Joined(p) | LineType::Unjoined(p) => p
        }
    }
}

pub fn connect_lines(lines: Vec<Line>, resolution: f32) -> (Vec<LineType>, QuadTree<Line>) {
    let (mut joined, qt) = join_lines(lines, resolution);

    loop {
        let mut any_progress = false;
        let (joined_t, p) = fuse_ends(joined);
        joined = joined_t;
        any_progress |= p;

        let (connected_t, p) = connect_linetypes(joined);
        joined = connected_t;
        any_progress |= p;

        if !any_progress {
            break;
        }
    }

    joined.retain(|lt| {
        match lt {
            &LineType::Joined(ref r) | &LineType::Unjoined(ref r) => r.len() > 1
        }
    });

    (joined, qt)
}

pub fn simplify_line(pts: Vec<Point>) -> Vec<Point> {
    if pts.len() <= 2 {
        return pts;
    }
    let mut pts = pts.into_iter();
    let mut out = vec![];

    let mut first = pts.next().unwrap();
    let mut prev = pts.next().unwrap();
    out.push(first);

    while let Some(p) = pts.next() {
        let line = Line(first, p);
        let dist_to_prev = line.dist_to_point(&prev);
        if dist_to_prev < EPSILON {
            prev = p;
        } else {
            out.push(prev);
            first = prev;
            prev = p;
        }
    }

    out.push(prev);
    out
}

fn connect_linetypes(mut lines: Vec<LineType>) -> (Vec<LineType>, bool) {
    fn overlap(a: Option<&Point>, b: Option<&Point>) -> bool {
        match (a, b) {
            (Some(a), Some(b)) => a.close_to(b, EPSILON),
            _ => false
        }
    }

    let mut made_progress = false;
    loop {
        let mut remove_this = None;
        'do_remove: for i in 0 .. lines.len() {
            for k in (i + 1) .. lines.len() {
                let (part_a, part_b) = lines.split_at_mut(i + 1);
                if let (&mut LineType::Unjoined(ref mut a),
                        &mut LineType::Unjoined(ref mut b)) = (&mut part_a[i], &mut part_b[k - i - 1]) {

                    // Aaaaaaaaaaa
                    // Bbbbbb
                    //  ->
                    // bbbbbAaaaaaaaaa
                    if overlap(a.first(), b.first()) {
                        b.reverse();
                        b.pop();
                        b.append(a);
                        remove_this = Some(i);
                        break 'do_remove;
                    }

                    // Aaaaaaaaaaa
                    // bbbbbbbB
                    //  ->
                    // bbbbbbbAaaaaaaaaaaa
                    if overlap(a.first(), b.last()) {
                        b.pop();
                        b.append(a);
                        remove_this = Some(i);
                        break 'do_remove;
                    }

                    // aaaaaaaaaaA
                    // Bbbbbb
                    //  ->
                    //  aaaaaaaaaBbbbbb
                    if overlap(a.last(), b.first()) {
                        a.pop();
                        a.append(b);
                        remove_this = Some(k);
                        break 'do_remove;
                    }
                    // aaaaaaaaA
                    // bbbbbbB
                    //  -> aaaaaaaaAbbbbbbb
                    if overlap(a.last(), b.last()) {
                        b.pop();
                        b.reverse();
                        a.append(b);
                        remove_this = Some(k);
                        break 'do_remove
                    }
                }
            }
        }

        if let Some(p) = remove_this {
            lines.swap_remove(p);
            made_progress = true;
        } else {
            break;
        }
    }

    (lines, made_progress)
}

fn fuse_ends(lines: Vec<LineType>) -> (Vec<LineType>, bool) {
    fn remove_dup(points: &mut Vec<Point>) {
        let first = points.first().cloned();
        let last = points.last().cloned();
        if let (Some(first), Some(last)) = (first, last) {
            if first.distance_2(&last) < EPSILON {
                points.pop();
            }
        }
    }

    let mut out = vec![];
    let mut made_progress = false;
    for line in lines {
        match line {
            LineType::Joined(mut points) => {
                let prev_len = points.len();
                remove_dup(&mut points);
                let post_len = points.len();
                if post_len < prev_len {
                    made_progress = true;
                }
                if post_len != 0 {
                    out.push(LineType::Joined(points));
                }
            },
            LineType::Unjoined(mut points) => {
                let prev_len = points.len();
                remove_dup(&mut points);
                let post_len = points.len();
                if post_len != 0 {
                    if post_len < prev_len {
                        made_progress = true;
                        out.push(LineType::Joined(points));
                    } else {
                        out.push(LineType::Unjoined(points));
                    }
                }
            }
        }
    }
    (out, made_progress)
}

fn join_lines(lines: Vec<Line>, resolution: f32) -> (Vec<LineType>, QuadTree<Line>) {
    let mut aabb: Option<Rect> = None;
    for line in &lines {
        if let Some(aabb) = aabb.as_mut() {
            *aabb = aabb.union_with(&line.bounding_box());
        }
        if aabb.is_none() {
            aabb = Some(line.bounding_box());
        }
    }

    let aabb = match aabb {
        Some(aabb) => aabb,
        None => return (vec![], QuadTree::new(Rect::null(), 4, 16, 4))
    };

    let mut tree = QuadTree::new(aabb, 4, 16, 4);
    for line in lines {
        tree.insert(line);
    }
    let tree_dup = tree.clone();

    let mut out = vec![];

    while !tree.is_empty() {
        let first_id = tree.first().unwrap();
        let (segment, _) = tree.remove(first_id).unwrap();
        let mut points = vec![segment.0, segment.1];
        let mut last = segment.1;

        loop {
            let closest = {
                let query = Rect::centered_with_radius(&last, resolution / 2.0);
                let mut near_last = tree.query(query);
                near_last.sort_by(|&(l1, _, _), &(l2, _, _)| {
                    let d1a = l1.0.distance_2(&last);
                    let d1b = l1.1.distance_2(&last);

                    let d2a = l2.0.distance_2(&last);
                    let d2b = l2.1.distance_2(&last);

                    let l1_min = d1a.min(d1b);
                    let l2_min = d2a.min(d2b);
                    l1_min.partial_cmp(&l2_min).unwrap_or(Ordering::Equal)
                });

                let closest_line_opt = near_last.into_iter().next();
                closest_line_opt.map(|(a, b, c)| {
                    (a.clone(), b.clone(), c.clone())
                })
            };

            if let Some((line, _, id)) = closest {
                tree.remove(id);
                if line.0.distance_2(&last) < line.1.distance_2(&last) {
                    last = line.1;
                    points.push(line.1);
                } else {
                    last = line.0;
                    points.push(line.0);
                }
            } else {
                break;
            }
        }

        out.push(LineType::Unjoined(points));
    }

    (out, tree_dup)
}
