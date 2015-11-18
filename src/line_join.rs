use std::cmp::{PartialOrd, Ordering};

use super::*;

pub enum LineType {
    Joined(Vec<Point>),
    Unjoined(Vec<Point>)
}

pub fn connect_lines(lines: Vec<Line>, resolution: f32) -> (Vec<LineType>, QuadTree<Line>) {
    let (mut joined, qt) = join_lines(lines, resolution);

    loop {
        let mut any_progress = false;
        let (joined_t, p) = fuse_ends(joined, resolution);
        joined = joined_t;
        any_progress |= p;

        let (connected_t, p) = connect_linetypes(joined, resolution);
        joined = connected_t;
        any_progress |= p;

        if !any_progress {
            break;
        }
    }

    (joined, qt)
}

fn connect_linetypes(lines: Vec<LineType>, _resolution: f32) -> (Vec<LineType>, bool) {
    (lines, false)
}

fn fuse_ends(lines: Vec<LineType>, resolution: f32) -> (Vec<LineType>, bool) {
    let mut out = vec![];
    let mut made_progress = false;
    for line in lines {
        match line {
            a@LineType::Joined(_) => out.push(a),
            LineType::Unjoined(mut points) => {
                let first = points.first().cloned().unwrap();
                let last = points.last().cloned().unwrap();
                if first.distance_2(&last) < resolution * resolution {
                    println!("first: {:?}, last: {:?}", first, last);
                    println!("dist: {}", first.distance_2(&last));
                    points.pop();
                    out.push(LineType::Joined(points));
                    made_progress = true;
                } else {
                    out.push(LineType::Unjoined(points));
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
                let query = Rect::centered_with_radius(&last, resolution);
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
                println!("{} found in query", near_last.len());

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
                println!("breaking");
                break;
            }
        }

        out.push(LineType::Unjoined(points));
    }

    (out, tree_dup)
}
