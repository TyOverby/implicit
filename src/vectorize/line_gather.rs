use ::Implicit;
use ::util::geom::{Line, Rect};
use super::{march, MarchResult};

pub trait QuadTreeProducer {
    type Tree;

    fn make_leaf_full(&mut self, rect: Rect) -> Self::Tree;
    fn make_leaf_empty(&mut self, rect: Rect) -> Self::Tree;
    fn make_leaf_line(&mut self, rect: Rect, fill: f32, line: Line) -> Self::Tree;
    fn make_leaf_double_line(&mut self, rect: Rect, fill: f32, l1: Line, l2: Line) -> Self::Tree;
    fn make_branch(&mut self, rect: Rect, a: Self::Tree, b: Self::Tree, c: Self::Tree, d: Self::Tree) -> Self::Tree;
    fn make_empty(&mut self, rect: Rect, fill: f32) -> Self::Tree;
}

impl QuadTreeProducer for () {
    type Tree = ();

    fn make_leaf_full(&mut self, _rect: Rect) -> Self::Tree {  }
    fn make_leaf_empty(&mut self, _rect: Rect) -> Self::Tree {  }
    fn make_leaf_line(&mut self, _rect: Rect, _fill: f32,  _line: Line) -> Self::Tree {  }
    fn make_leaf_double_line(&mut self, _rect: Rect, _fill: f32, _l1: Line, _l2: Line) -> Self::Tree {  }
    fn make_branch(&mut self, _rect: Rect, _a: Self::Tree, _b: Self::Tree, _c: Self::Tree, _d: Self::Tree) -> Self::Tree {  }
    fn make_empty(&mut self, _rect: Rect, _fill: f32) -> Self::Tree {  }
}

#[inline]
fn should_early_return<P: QuadTreeProducer>(p: &mut P, v: f32, rect: Rect) -> Option<P::Tree> {
    let furthest = {
        let r = rect.width() / 2.0;
        2.0 * (r * r)
    };

    if v * v > furthest {
        Some(if v < 0.0 {
            p.make_leaf_full(rect)
        } else {
            p.make_leaf_empty(rect)
        })
    } else {
        None
    }
}

fn gather_final<S: ?Sized, P>(p: &mut P, shape: &S, rect: Rect, out: &mut Vec<Line>, a: f32, b: f32, c: f32, d: f32) -> P::Tree
where S: Implicit, P: QuadTreeProducer {
    let midpoint = rect.midpoint();
    let m = shape.sample(midpoint);

    if let Some(early_return) = should_early_return(p, m, rect) {
        return early_return;
    }

    let result = march(a, b, c, d, m, midpoint, rect.width());
    match result {
        MarchResult::One(l) => {
            out.push(l);
            p.make_leaf_line(rect, m, l)
        }
        MarchResult::Two(l1, l2) => {
            out.push(l1);
            out.push(l2);
            p.make_leaf_double_line(rect, m, l1, l2)
        }
        _ => {
            p.make_empty(rect, m)
        },
    }
}

// A N B
// W M E
// D S C
fn gather<S: ?Sized, P: QuadTreeProducer>(p: &mut P, shape: &S, rect: Rect, depth: u32, out: &mut Vec<Line>) -> P::Tree
where S: Implicit, P: QuadTreeProducer {
    let midpoint = rect.midpoint();
    let m = shape.sample(midpoint);

    if let Some(early_return) = should_early_return(p, m, rect) {
        return early_return;
    }

    let nw_quad = Rect::from_points(&rect.top_left(), &midpoint);
    let ne_quad = Rect::from_points(&rect.top_right(), &midpoint);
    let se_quad = Rect::from_points(&rect.bottom_right(), &midpoint);
    let sw_quad = Rect::from_points(&rect.bottom_left(), &midpoint);

    if depth == 1 {
        let (north, south, east, west) = (rect.north(), rect.south(), rect.east(), rect.west());

        ::flame::start("sampling");
        let a = shape.sample(rect.top_left());
        let b = shape.sample(rect.top_right());
        let c = shape.sample(rect.bottom_right());
        let d = shape.sample(rect.bottom_left());

        let n = shape.sample(north);
        let s = shape.sample(south);
        let e = shape.sample(east);
        let w = shape.sample(west);
        ::flame::end("sampling");

        let xa = gather_final(p, shape, nw_quad, out, a, n, m, w);
        let xb = gather_final(p, shape, ne_quad, out, n, b, e, m);
        let xc = gather_final(p, shape, se_quad, out, m, e, c, s);
        let xd = gather_final(p, shape, sw_quad, out, w, m, s, d);

        p.make_branch(rect, xa, xb, xc, xd)
    } else {
        let xa = gather(p, shape, nw_quad, depth - 1, out);
        let xb = gather(p, shape, ne_quad, depth - 1, out);
        let xc = gather(p, shape, se_quad, depth - 1, out);
        let xd = gather(p, shape, sw_quad, depth - 1, out);

        p.make_branch(rect, xa, xb, xc, xd)
    }
}

pub fn gather_lines<S: ?Sized, P>(p: &mut P, shape: &S, depth: u32) -> (P::Tree, Vec<Line>)
where S: Implicit + Sync, P: QuadTreeProducer {
    let mut out = vec![];

    let depth = if depth == 0 { 1 } else { depth };

    let bb = shape.bounding_box().unwrap();
    let w = bb.width();
    let h = bb.height();
    let w = w + w / 6.0;
    let h = h + h / 6.0;
    let bounding_box = Rect::centered_with_radius(&bb.midpoint(), w.max(h));

    let tree = gather(p, shape, bounding_box, depth, &mut out);
    (tree, out)
}
