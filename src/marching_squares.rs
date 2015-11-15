use Implicit;
use geom::{Line, Point, Vector};

pub enum MarchResult {
    None,
    One(Line),
    Two(Line, Line)
}


//
// A N B
// W P E
// D S C
const A: Vector = Vector {x: -0.5, y: -0.5};
const B: Vector = Vector {x:  0.5, y: -0.5};
const C: Vector = Vector {x:  0.5, y:  0.5};
const D: Vector = Vector {x:  0.5, y:  0.5};

pub fn march<I: Implicit>(i: &I, p: Point, dist: f32) -> MarchResult {
    let p1 = A * dist + p;
    let p2 = C * dist + p;
    MarchResult::One(Line(p1, p2))
}
