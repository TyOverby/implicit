use super::{
    Point,
    Rect,
};
use ::Implicit;
use std::cmp::{PartialOrd, Ordering};
use itertools::Itertools;
use flame;

#[derive(Copy, Clone, Debug)]
struct SampleDist {
    pub x_bump: f32,
    pub y_bump: f32,
}

enum PoorMansQuadTree {
    Node {
        bb: Rect,
        children: [Option<Box<PoorMansQuadTree>>; 4],
    },
    Leaf(Rect)
}

impl PoorMansQuadTree {
    fn could_contain(&self, point: Point) -> bool {
        match self {
            &PoorMansQuadTree::Node { bb, .. } => bb.contains(&point),
            &PoorMansQuadTree::Leaf(rect) => rect.contains(&point),
        }
    }
    fn contains(&self, point: Point) -> bool {
        match self {
            &PoorMansQuadTree::Node { ref bb, ref children } => {
                if !bb.contains(&point) {
                    false
                } else {
                    for child in (&children[..]).iter().filter_map(|a| a.as_ref()) {
                        if child.could_contain(point) {
                            return child.contains(point);
                        }
                    }
                    false
                }
            }
            &PoorMansQuadTree::Leaf(rect) => {
                rect.contains(&point)
            }
        }
    }

    fn is_leaf(&self) -> bool {
        match self {
            &PoorMansQuadTree::Leaf(_) => true,
            _ => false
        }
    }

    fn build<I: Implicit>(shape: &I, bb: Rect, sample_dist: SampleDist) -> Option<PoorMansQuadTree> {
        let radius = bb.width().max(bb.height());
        let sample = shape.sample(bb.midpoint()).abs();

        if sample > radius { return None; }
        if radius < sample_dist.max_bump() * 10.0 || radius < 1.0 {
            return Some(PoorMansQuadTree::Leaf(bb));
        }

        let q = bb.split_quad();
        let a = PoorMansQuadTree::build(shape, q[0], sample_dist);
        let b = PoorMansQuadTree::build(shape, q[1], sample_dist);
        let c = PoorMansQuadTree::build(shape, q[2], sample_dist);
        let d = PoorMansQuadTree::build(shape, q[3], sample_dist);

        let a_leaf = a.as_ref().map(|pmqt| pmqt.is_leaf()).unwrap_or(false);
        let b_leaf = b.as_ref().map(|pmqt| pmqt.is_leaf()).unwrap_or(false);
        let c_leaf = c.as_ref().map(|pmqt| pmqt.is_leaf()).unwrap_or(false);
        let d_leaf = d.as_ref().map(|pmqt| pmqt.is_leaf()).unwrap_or(false);

        if a_leaf && b_leaf && c_leaf && d_leaf {
            return Some(PoorMansQuadTree::Leaf(bb));
        }

        return Some(PoorMansQuadTree::Node {
            bb: bb,
            children: [a.map(Box::new), b.map(Box::new), c.map(Box::new), d.map(Box::new)]
        })
    }
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


pub fn sampling_points<S: Implicit>(shape: &S, resolution: f32) -> Vec<(f32, f32)> {
    let bb = shape.bounding_box().unwrap();
    let b_dim = bb.width().max(bb.height());
    let expand = b_dim * 0.10;
    let bb = bb.expand(expand, expand, expand, expand);

    ::flame::start("build poor mans quad tree");
    let sample_dist = SampleDist { x_bump: resolution, y_bump: resolution };
    let pmqt = PoorMansQuadTree::build(shape, bb, sample_dist).unwrap();
    ::flame::end("build poor mans quad tree");

    let mut out = vec![];
    ::flame::start("filter points");
    for p in sample_from_box(bb, sample_dist) {
        if pmqt.contains(p) {
            out.push((p.x, p.y));
        }
    }
    ::flame::end("filter points");

    out
}

fn sample_from_box(mut bb: Rect, sample_dist: SampleDist) -> BoxSampler {
    sample_dist.modify_bb(&mut bb);
    let Point{x, y} = bb.top_left();
    let x_orig = x;
    BoxSampler {
        x: x,
        y: y,
        bb: bb,
        x_orig: x_orig,
        sample_dist: sample_dist
    }
}

struct BoxSampler {
    x: f32,
    y: f32,
    bb: Rect,
    x_orig: f32,
    sample_dist: SampleDist,
}

impl Iterator for BoxSampler {
    type Item = Point;
    fn next(&mut self) -> Option<Point> {
        let p = Point{x: self.x, y: self.y};
        if self.bb.contains(&p) {
            self.x = self.sample_dist.bump_x(self.x);
            Some(p)
        } else {
            self.x = self.x_orig;
            self.y = self.sample_dist.bump_y(self.y);
            if !self.bb.contains(&Point{x: self.x, y: self.y}) {
                None
            } else {
                self.next()
            }
        }
    }
}

