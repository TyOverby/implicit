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

fn gather<S: ?Sized, P>(p: &mut P, shape: &S, rect: Rect, depth: u32, out: &mut Vec<Line>) -> P::Tree
where S: Implicit, P: QuadTreeProducer {
    let a = shape.sample(rect.top_left());
    let b = shape.sample(rect.top_right());
    let c = shape.sample(rect.bottom_right());
    let d = shape.sample(rect.bottom_left());
    gather_real(p, shape, rect, depth, out, a, b, c, d)
}

// A N B
// W M E
// D S C
fn gather_real<S: ?Sized, P: QuadTreeProducer>(p: &mut P, shape: &S, rect: Rect, depth: u32, out: &mut Vec<Line>, a: f32, b: f32, c: f32, d: f32) -> P::Tree
where S: Implicit, P: QuadTreeProducer {
    let midpoint = rect.midpoint();
    let m = shape.sample(midpoint);

    let furthest = {
        let r = rect.width() / 2.0;
        let _2r2 = 2.0 * (r * r);
        _2r2.sqrt()
    };
    if m.abs() > furthest {
        if m < 0.0 {
            return p.make_leaf_full(rect)
        } else {
            return p.make_leaf_empty(rect)
        }
    }

    if depth == 0 {
        let result = march(a, b, c, d, Some(m), shape, midpoint, rect.width());
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
    } else {
        let (north, south, east, west) = (rect.north(), rect.south(), rect.east(), rect.west());
        let n = shape.sample(north);
        let s = shape.sample(south);
        let e = shape.sample(east);
        let w = shape.sample(west);

        let nw_quad = Rect::from_points(&rect.top_left(), &midpoint);
        let ne_quad = Rect::from_points(&rect.top_right(), &midpoint);
        let se_quad = Rect::from_points(&rect.bottom_right(), &midpoint);
        let sw_quad = Rect::from_points(&rect.bottom_left(), &midpoint);

        let xa = gather_real::<_, P>(p, shape, nw_quad, depth - 1, out, a, n, m, w);
        let xb = gather_real::<_, P>(p, shape, ne_quad, depth - 1, out, n, b, e, m);
        let xc = gather_real::<_, P>(p, shape, se_quad, depth - 1, out, m, e, c, s);
        let xd = gather_real::<_, P>(p, shape, sw_quad, depth - 1, out, w, m, s, d);

        p.make_branch(rect, xa, xb, xc, xd)
    }
}

pub fn gather_lines_2<S: ?Sized, P>(p: &mut P, shape: &S, depth: u32) -> (P::Tree, Vec<Line>)
where S: Implicit + Sync, P: QuadTreeProducer {
    let mut out = vec![];

    let bb = shape.bounding_box().unwrap();
    let w = bb.width();
    let h = bb.height();
    let w = w + w / 6.0;
    let h = h + h / 6.0;
    let bounding_box = Rect::centered_with_radius(&bb.midpoint(), w.max(h));

    let tree = gather(p, shape, bounding_box, depth, &mut out);
    (tree, out)
}
