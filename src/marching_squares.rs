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

fn n(distance: f32, point: Point) -> Point {
    N * distance + point
}

fn s(distance: f32, point: Point) -> Point {
    S * distance + point
}

fn e(distance: f32, point: Point) -> Point {
    E * distance + point
}

fn w(distance: f32, point: Point) -> Point {
    W * distance + point
}


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
            MarchResult::One(Line(w(dist, p), s(dist, p)))
        },
        // o o
        // o .
        (false, false, true, false)  => {
            MarchResult::One(Line(s(dist, p), e(dist, p)))
        },
        // o o
        // . .
        (false, false, true, true)  => {
            MarchResult::One(Line(w(dist, p), e(dist, p)))
        },
        // o .
        // o o
        (false, true, false, false)  => {
            MarchResult::One(Line(n(dist, p), e(dist, p)))
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
                    Line(w(dist, p), n(dist, p)),
                    Line(s(dist, p), e(dist, p)))
            }
            // o   .
            //   o
            // .   o
            else {
                MarchResult::Two(
                    Line(w(dist, p), s(dist, p)),
                    Line(n(dist, p), e(dist, p)))
            }
        },
        // o .
        // o .
        (false, true, true, false)  => {
            MarchResult::One(Line(n(dist, p), s(dist, p)))
        },
        // o .
        // . .
        (false, true, true, true)  => {
            MarchResult::One(Line(w(dist, p), n(dist, p)))
        },
        // . o
        // o o
        (true, false, false, false)  => {
            MarchResult::One(Line(w(dist, p), n(dist, p)))
        },
        // . o
        // . o
        (true, false, false, true)  => {
            MarchResult::One(Line(n(dist, p), s(dist, p)))
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
                    Line(n(dist, p), e(dist, p)),
                    Line(w(dist, p), s(dist, p)))
            }
            // .   o
            //   o
            // o   .
            else {
                MarchResult::Two(
                    Line(w(dist, p), n(dist, p)),
                    Line(s(dist, p), e(dist, p)))
            }
        },
        // . o
        // . .
        (true, false, true, true)  => {
            MarchResult::One(Line(n(dist, p), e(dist, p)))
        },

        // . .
        // o o
        (true, true, false, false)  => {
            MarchResult::One(Line(w(dist, p), e(dist, p)))
        },
        // . .
        // . o
        (true, true, false, true)  => {
            MarchResult::One(Line(s(dist, p), e(dist, p)))
        },

        // . .
        // o .
        (true, true, true, false)  => {
            MarchResult::One(Line(w(dist, p), s(dist, p)))
        },
        // . .
        // . .
        (true, true, true, true)  => {
            MarchResult::None
        },
    }
}
