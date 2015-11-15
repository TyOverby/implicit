use Implicit;
use geom::{Line, Point, Vector};

pub enum MarchResult {
    None,
    One(Line),
    OneDebug(Line),
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

#[inline(always)]
fn n(distance: f32, point: Point, how_much: f32) -> Point {
    let mut result = N * distance + point;
    result.x += how_much;
    result
}

#[inline(always)]
fn s(distance: f32, point: Point, how_much: f32) -> Point {
    let mut result = S * distance + point;
    result.x += how_much;
    result
}

#[inline(always)]
fn e(distance: f32, point: Point, how_much: f32) -> Point {
    let mut result = E * distance + point;
    result.y += how_much;
    result
}

#[inline(always)]
fn w(distance: f32, point: Point, how_much: f32) -> Point {
    let mut result = W * distance + point;
    result.y += how_much;
    result
}

fn lerp(fa: f32, fb: f32, dist: f32) -> f32 {
    -dist / 2.0 + dist * ((-fa) / (fb - fa))
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
            let da = lerp(sra, srd, dist);
            let dc = lerp(src, srd, dist);
            MarchResult::One(Line(w(dist, p, da), s(dist, p, -dc)))
        },
        // o o
        // o .
        (false, false, true, false)  => {
            let db = lerp(src, srb, dist);
            let dd = lerp(srd, src, dist);
            MarchResult::One(Line(s(dist, p, dd), e(dist, p, -db)))
        },
        // o o
        // . .
        (false, false, true, true)  => {
            let da = lerp(sra, srd, dist);
            let db = lerp(srb, src, dist);
            MarchResult::One(Line(w(dist, p, da), e(dist, p, db)))
        },
        // o .
        // o o
        (false, true, false, false)  => {
            let da = lerp(sra, srb, dist);
            let db = lerp(srb, src, dist);
            MarchResult::One(Line(n(dist, p, da), e(dist, p, db)))
        },
        // o .
        // . o
        // TODO: linear interpolation here.
        (false, true, false, true)  => {
            let srm = i.sample(p);
            let m_on = srm <= 0.0;

            // o   .
            //   .
            // .   o
            if m_on {
                MarchResult::Two(
                    Line(w(dist, p, 0.0), n(dist, p, 0.0)),
                    Line(s(dist, p, 0.0), e(dist, p, 0.0)))
            }
            // o   .
            //   o
            // .   o
            else {
                MarchResult::Two(
                    Line(w(dist, p, 0.0), s(dist, p, 0.0)),
                    Line(n(dist, p, 0.0), e(dist, p, 0.0)))
            }
        },
        // o .
        // o .
        (false, true, true, false)  => {
            let da = lerp(srb, sra, dist);
            let dd = lerp(src, srd, dist);
            MarchResult::One(Line(n(dist, p, -da), s(dist, p, -dd)))
        },
        // o .
        // . .
        (false, true, true, true)  => {
            let dc = lerp(sra, srb, dist);
            let dd = lerp(sra, srd, dist);
            MarchResult::One(Line(w(dist, p, dd), n(dist, p, dc)))
        },
        // . o
        // o o
        (true, false, false, false)  => {

            MarchResult::OneDebug(Line(w(dist, p, 0.0), n(dist, p, 0.0)))
        },
        // . o
        // . o
        (true, false, false, true)  => {
            MarchResult::One(Line(n(dist, p, 0.0), s(dist, p, 0.0)))
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
                    Line(n(dist, p, 0.0), e(dist, p, 0.0)),
                    Line(w(dist, p, 0.0), s(dist, p, 0.0)))
            }
            // .   o
            //   o
            // o   .
            else {
                MarchResult::Two(
                    Line(w(dist, p, 0.0), n(dist, p, 0.0)),
                    Line(s(dist, p, 0.0), e(dist, p, 0.0)))
            }
        },
        // . o
        // . .
        (true, false, true, true)  => {
            let da = lerp(sra, srb, dist);
            let dc = lerp(src, srb, dist);
            MarchResult::One(Line(n(dist, p, da), e(dist, p, -dc)))
        },

        // . .
        // o o
        (true, true, false, false)  => {
            MarchResult::One(Line(w(dist, p, 0.0), e(dist, p, 0.0)))
        },
        // . .
        // . o
        (true, true, false, true)  => {
            MarchResult::One(Line(s(dist, p, 0.0), e(dist, p, 0.0)))
        },

        // . .
        // o .
        (true, true, true, false)  => {
            MarchResult::One(Line(w(dist, p, 0.0), s(dist, p, 0.0)))
        },
        // . .
        // . .
        (true, true, true, true)  => {
            MarchResult::None
        },
    }
}
