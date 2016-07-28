use super::{
    Point,
    Rect,
    Bitmap,
};
use ::{Implicit, QuadTree};

#[derive(Copy, Clone, Debug)]
struct SampleDist {
    pub x_bump: f32,
    pub y_bump: f32,
}

#[derive(Debug)]
enum PmQuadTree {
    Node {
        bb: Rect,
        children: [Option<Box<PmQuadTree>>; 4],
    },
    Leaf(Rect)
}
use self::PmQuadTree::*;

impl PmQuadTree {
    fn could_contain(&self, point: Point) -> bool {
        match self {
            &Node { bb, .. } => bb.contains(&point),
            &Leaf(rect) => rect.contains(&point),
        }
    }

    #[inline(always)]
    fn contains(&self, point: Point) -> bool {
        match self {
            &Node { ref bb, ref children } => {
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
            &Leaf(rect) => {
                rect.contains(&point)
            }
        }
    }

    fn is_leaf(&self) -> bool {
        match self {
            &Leaf(_) => true,
            _ => false
        }
    }
    fn is_node(&self) -> bool {
        !self.is_leaf()
    }

    fn build<I: Implicit>(shape: &I, bb: Rect, sample_dist: SampleDist) -> Option<(PmQuadTree, bool)> {
        let radius_max = bb.width().max(bb.height());
        let radius_min = bb.width().min(bb.height());
        let sample = shape.sample(bb.midpoint()).abs();

        if sample > radius_max {
            return None;
        } else if radius_min < sample_dist.max_bump() * 5.0 {
            return Some((Leaf(bb), true));
        }

        let q = bb.split_quad();
        let a = PmQuadTree::build(shape, q[0], sample_dist);
        let b = PmQuadTree::build(shape, q[1], sample_dist);
        let c = PmQuadTree::build(shape, q[2], sample_dist);
        let d = PmQuadTree::build(shape, q[3], sample_dist);

        let (a, b, c, d) = match (a, b, c, d) {
            (Some((Leaf(_), true)), Some((Leaf(_), true)),
             Some((Leaf(_), true)), Some((Leaf(_), true))) => return Some((Leaf(bb), true)),

            (Some((Leaf(a), true)), Some((Leaf(b), true)), None, None) => return Some((Leaf(a.union_with(&b)), false)),
            (None, None, Some((Leaf(c), true)), Some((Leaf(d), true))) => return Some((Leaf(c.union_with(&d)), false)),
            (Some((Leaf(a), true)), None, Some((Leaf(c), true)), None) => return Some((Leaf(a.union_with(&c)), false)),
            (None, Some((Leaf(b), true)), None, Some((Leaf(d), true))) => return Some((Leaf(b.union_with(&d)), false)),

            (Some((a, _)), None, None, None) => return Some((a, false)),
            (None, Some((b, _)), None, None) => return Some((b, false)),
            (None, None, Some((c, _)), None) => return Some((c, false)),
            (None, None, None, Some((d, _))) => return Some((d, false)),
            (None, None, None, None) => return None,
            (a, b, c, d) => (a, b, c, d),
        };

        return Some((Node {
            bb: bb,
            children: [
                a.map(|p| Box::new(p.0)),
                b.map(|p| Box::new(p.0)),
                c.map(|p| Box::new(p.0)),
                d.map(|p| Box::new(p.0)),
            ]
        }, false))
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


pub fn sampling_points<S: Implicit>(shape: &S, resolution: f32) -> Vec<Point> {
    fn sample_with_pmqt(rect: Rect, sample_dist: SampleDist, pmqt: &PmQuadTree) -> (Vec<Point>, Vec<Point>) {
        let mut out_uncontested = vec![];
        let mut out_contested = vec![];
        for (p, c) in sample_from_box(rect, sample_dist) {
            match (pmqt.contains(p), c) {
                (true, true) => out_uncontested.push(p),
                (true, false) => out_contested.push(p),
                (false, _) => {}
            }
        }

        (out_uncontested, out_contested)
    }

    fn sample_with_bitmap(rect: Rect, sample_dist: SampleDist, bitmap: &Bitmap) -> (Vec<Point>, Vec<Point>) {
        let mut out_uncontested = vec![];
        let mut out_contested = vec![];
        for (p, c) in sample_from_box(rect, sample_dist) {
            let Point{x, y} = p;
            let sample = bitmap.sample(x, y, |a, b, c, d| (a.abs()).min(b.abs()).min(c.abs()).min(d.abs()));

            let cmp = sample_dist.max_bump() * 5.0;
            let bll = sample < cmp;

            match (bll, c) {
                (true, true) => {
                    out_uncontested.push(p);
                }
                (true, false) =>  {
                    out_contested.push(p);
                }
                (false, _) => {}
            }
        }

        (out_uncontested, out_contested)
    }

    let bb = shape.bounding_box().unwrap();
    let expand = resolution * 2.0;
    let bb = bb.expand(expand, expand, expand, expand);

    ::flame::start("build poor mans quad tree");
    let sample_dist = SampleDist { x_bump: resolution, y_bump: resolution };
    //let (pmqt, _) = PmQuadTree::build(shape, bb, sample_dist).unwrap();
    ::flame::end("build poor mans quad tree");

    ::flame::start("build bitmap");
    let bitmap = Bitmap::new(shape, sample_dist.max_bump() * 5.0);
    println!("bitmap size: {:?}", bitmap.size());
    ::flame::end("build bitmap");

    ::flame::start("filter sample with bitmap");
    let (mut out, to_filter) = sample_with_bitmap(bb, sample_dist, &bitmap);
    println!("out: {}, to_filter: {}", out.len(), to_filter.len());
    ::flame::end("filter sample with bitmap");

    ::flame::start("filter points");
    let mut quadtree = QuadTree::new(bb, false,  5, 20, 5);
    for contested in to_filter {
        quadtree.insert(contested);
    }
    for (_, &(ok, _)) in quadtree.iter() {
        out.push(ok);
    }
    ::flame::end("filter points");

    /*
    println!("{:#?}", pmqt);
    println!("{} sampling points", out.len());
    */

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
        sample_dist: sample_dist,
        on_top: true,
    }
}

struct BoxSampler {
    x: f32,
    y: f32,
    bb: Rect,
    x_orig: f32,
    sample_dist: SampleDist,
    on_top: bool,
}

impl Iterator for BoxSampler {
    type Item = (Point, bool);
    fn next(&mut self) -> Option<(Point, bool)> {
        let p = Point{x: self.x, y: self.y};
        let on_left = p.x == self.x_orig;
        let on_right = self.sample_dist.bump_x(self.x) > self.bb.right();
        let on_top = self.on_top;
        let on_bottom = self.sample_dist.bump_y(self.y) > self.bb.bottom();

        if self.bb.contains(&p) {
            self.x = self.sample_dist.bump_x(self.x);
            Some((p, !(on_left || on_right || on_top || on_bottom)))
        } else {
            self.x = self.x_orig;
            self.y = self.sample_dist.bump_y(self.y);
            self.on_top = false;
            if !self.bb.contains(&Point{x: self.x, y: self.y}) {
                None
            } else {
                self.next()
            }
        }
    }
}

