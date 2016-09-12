use ::vectorize::line_gather::QuadTreeProducer;
use ::util::geom::{Rect, Line};
use super::SampleValue;

pub struct DrawWrapper<'a>(pub &'a mut FnMut(Rect, SampleValue));

impl <'a> QuadTreeProducer for DrawWrapper<'a> {
    type Tree = ();

    fn make_leaf_full(&mut self, rect: Rect) -> Self::Tree {
        self.0(rect, SampleValue::Filled)
    }

    fn make_leaf_empty(&mut self, rect: Rect) -> Self::Tree {
        self.0(rect, SampleValue::Empty)
    }

    fn make_leaf_line(&mut self, rect: Rect, fill: f32, _: Line) -> Self::Tree {
        self.0(rect, SampleValue::Partial(fill))
    }

    fn make_leaf_double_line(&mut self, rect: Rect, fill: f32, _l1: Line, _l2: Line) -> Self::Tree {
        self.0(rect, SampleValue::Partial(fill))
    }
    fn make_branch(&mut self, _rect: Rect, _a: Self::Tree, _b: Self::Tree, _c: Self::Tree, _d: Self::Tree) -> Self::Tree {
    }

    fn make_empty(&mut self, rect: Rect, fill: f32) -> Self::Tree {
        self.0(rect, SampleValue::Partial(fill))
    }
}