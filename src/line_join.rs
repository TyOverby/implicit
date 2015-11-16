use std::cmp::{PartialOrd, Ordering};

use super::*;

pub enum LineType {
    Joined(Vec<Point>),
    Unjoined(Vec<Point>)
}

pub fn join_lines(lines: Vec<Line>, resolution: f32) -> Vec<LineType> {
    let mut aabb: Option<Rect> = None;
    for line in &lines {
        let mut set = false;
        if let Some(aabb) = aabb.as_mut() {
            aabb.union_with(&line.bounding_box());
            set = true;
        }
        if !set {
            aabb = Some(line.bounding_box());
        }
    }
    let aabb = match aabb {
        Some(aabb) => aabb,
        None => return vec![]
    };

    let mut tree = QuadTree::new(aabb, 4, 16, 4);
    for line in lines {
        tree.insert(line);
    }

    while !tree.is_empty() {
        let first_id = tree.first().unwrap();
        let (segment, _) = tree.remove(first_id).unwrap();
        let mut points = vec![segment.0, segment.1];
        let mut last = segment.1;

        loop {
            let closest = {
                let mut near_last = tree.query(Rect::centered_with_radius(&last, resolution));
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

        // TODO:
        // group all the points together in a line.  Check to see if it's closed,
    }

    unimplemented!();
}
