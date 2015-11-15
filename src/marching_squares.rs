use Implicit;
use geom::{Line, Point, Vector};

pub enum MarchResult {
    None,
    One(Line),
    Two(Line, Line),
    Debug
}

// A N B
// W P E
// D S C
const A: Vector = Vector { x: -0.5, y: -0.5 };
const B: Vector = Vector { x:  0.5, y: -0.5 };
const C: Vector = Vector { x:  0.5, y:  0.5 };
const D: Vector = Vector { x: -0.5, y:  0.5 };

const N: Vector = Vector { x:  0.0, y: -0.5 };
const S: Vector = Vector { x:  0.0, y:  0.5 };
const E: Vector = Vector { x:  0.5, y:  0.0 };
const W: Vector = Vector { x: -0.5, y:  0.0 };

pub fn march<I: Implicit>(i: &I, p: Point, dist: f32) -> MarchResult {
    let sa = A * dist + p;
    let sb = B * dist + p;
    let sc = C * dist + p;
    let sd = D * dist + p;

    let sra = i.sample(sa);
    let srb = i.sample(sb);
    let src = i.sample(sc);
    let srd = i.sample(sd);

    let a_on = sra <= 0.0;
    let b_on = srb <= 0.0;
    let c_on = src <= 0.0;
    let d_on = srd <= 0.0;

    match (a_on, b_on, c_on, d_on) {
        // o o
        // o o
        (false, false, false, false) => MarchResult::None,
        // o o
        // . o
        (false, false, false, true)  => {
            MarchResult::One(Line(W * dist + p, S * dist + p))
        },
        // o o
        // o .
        (false, false, true, false)  => {
            MarchResult::One(Line(S * dist + p, E * dist + p))
        },
        // o o
        // . .
        (false, false, true, true)  => {
            MarchResult::One(Line(W * dist + p, E * dist + p))
        },
        // o .
        // o o
        (false, true, false, false)  => {
            MarchResult::One(Line(N * dist + p, E * dist + p))
        },
        // o .
        // . o
        (false, true, false, true)  => {
            let srm = i.sample(p);
            let m_on = srm <= 0.0;

            // o   .
            //   .
            // .   o
            if m_on {
                MarchResult::Two(
                    Line(W * dist + p, N * dist + p),
                    Line(S * dist + p, E * dist + p))
            }
            // o   .
            //   o
            // .   o
            else {
                MarchResult::Two(
                    Line(W * dist + p, S * dist + p),
                    Line(N * dist + p, E * dist + p))
            }
        },
        // o .
        // o .
        (false, true, true, false)  => {
            MarchResult::One(Line(N * dist + p, S * dist + p))
        },
        // o .
        // . .
        (false, true, true, true)  => {
            MarchResult::One(Line(W * dist + p, N * dist + p))
        },
        // . o
        // o o
        (true, false, false, false)  => {
            MarchResult::One(Line(W * dist + p, N * dist + p))
        },
        // . o
        // . o
        (true, false, false, true)  => {
            MarchResult::One(Line(N * dist + p, S * dist + p))
        },
        // . o
        // o .
        (true, false, true, false)  => {
            let srm = i.sample(p);
            let m_on = srm <= 0.0;

            // .   o
            //   .
            // o   .
            if m_on {
                MarchResult::Two(
                    Line(N * dist + p, E * dist + p),
                    Line(W * dist + p, S * dist + p))
            }
            // .   o
            //   o
            // o   .
            else {
                MarchResult::Two(
                    Line(W * dist + p, N * dist + p),
                    Line(S * dist + p, E * dist + p))
            }
        },
        // . o
        // . .
        (true, false, true, true)  => {
            MarchResult::One(Line(N * dist + p, E * dist + p))
        },

        // . .
        // o o
        (true, true, false, false)  => {
            MarchResult::One(Line(W * dist + p, E * dist + p))
        },
        // . .
        // . o
        (true, true, false, true)  => {
            MarchResult::One(Line(S * dist + p, E * dist + p))
        },

        // . .
        // o .
        (true, true, true, false)  => {
            MarchResult::One(Line(W * dist + p, S * dist + p))
        },
        // . .
        // . .
        (true, true, true, true)  => {
            MarchResult::None
        },
    }
}
