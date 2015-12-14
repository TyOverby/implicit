extern crate lux;
extern crate implicit;

mod helper;

use implicit::*;

const NECK_CIRC: f32 = 14.5;
const FRONT_LEN: f32 = (3.0 / 4.0) * NECK_CIRC;
const MAIN_HEIGHT: f32 = 1.0;
const TRI_OFFSET: f32 = 0.5;

// holes
const HOLE_RADIUS: f32 = 0.125;
const HOLE_SPACING: f32 = 0.25 + HOLE_RADIUS * 2.0;
const HOLE_OFFSET: f32 = 0.5;
const STITCH_OFFSET: f32 = 0.1;

fn front_outline() -> OrThese<Polygon> {
    let main_front_rect = Rectangle::new(Rect::from_points(
            &Point {x: 0.0, y: 0.0},
            &Point {x: FRONT_LEN, y: MAIN_HEIGHT}));

    let left_triangle = Polygon::new(vec![
            Point { x: -TRI_OFFSET, y: MAIN_HEIGHT / 2.0 },
            Point { x: 0.0, y: 0.0 },
            Point { x: 0.0, y: MAIN_HEIGHT }].into_iter());

    let right_triangle = Polygon::new(vec![
            Point { x: FRONT_LEN + TRI_OFFSET, y: MAIN_HEIGHT / 2.0 },
            Point { x: FRONT_LEN, y: 0.0 },
            Point { x: FRONT_LEN, y: MAIN_HEIGHT }].into_iter());

    OrThese::combine(0.1, vec![
        Box::new(main_front_rect) as Box<Implicit>,
        Box::new(left_triangle.clone()) as Box<Implicit>,
        Box::new(right_triangle.clone()) as Box<Implicit>,
    ])
}

fn holes() -> Vec<Box<implicit::Implicit>> {
    let mut holes = vec![];
    for i in 0 .. 4 {
        holes.push(
            Circle {
                center: Point {
                    x: FRONT_LEN - HOLE_OFFSET - HOLE_SPACING * i as f32,
                    y: MAIN_HEIGHT / 2.0
                },
                radius: HOLE_RADIUS
            }.not().boxed());
        holes.push(
            Circle {
                center: Point {
                    x: HOLE_OFFSET + HOLE_SPACING * i as f32,
                    y: MAIN_HEIGHT / 2.0
                },
                radius: HOLE_RADIUS
            }.not().boxed());
    }
    holes
}

fn main() {
    let front_collar = front_outline();
    let outline_stitch = front_collar.clone().shrink(STITCH_OFFSET);
    let holes = holes();

    let mut targets = holes;
    targets.push(front_collar.boxed());
    let front_collar = AndThese { targets: targets };

    let f = GenericShape::Boxed(Box::new(front_collar));

    helper::display(vec![
        (&f.scale(100.0, 100.0).translate(50.0, 50.0),              helper::Display::Lines),
        (&outline_stitch.scale(100.0, 100.0).translate(50.0, 50.0), helper::Display::Lines),
                    ]);
}
