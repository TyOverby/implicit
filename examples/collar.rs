extern crate lux;
extern crate implicit;

mod helper;

use implicit::*;
use implicit::geom::*;

const NECK_CIRC: f32 = 14.5;
const FRONT_LEN: f32 = (3.0 / 4.0) * NECK_CIRC;
const MAIN_HEIGHT: f32 = 1.0;
const TRI_OFFSET: f32 = 0.5;

// holes
const HOLE_RADIUS: f32 = 0.125;
const HOLE_SPACING: f32 = 0.25 + HOLE_RADIUS * 2.0;
const HOLE_OFFSET: f32 = 0.5;
const STITCH_OFFSET: f32 = 0.1;

fn front_outline() -> PolyGroup {
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


    main_front_rect.or(left_triangle).or(right_triangle).fix_rules(0.01)
}

fn holes() -> Vec<Not<Circle>> {
    let mut holes = vec![];
    for i in 0 .. 4 {
        holes.push(
            Circle {
                center: Point {
                    x: FRONT_LEN - HOLE_OFFSET - HOLE_SPACING * i as f32,
                    y: MAIN_HEIGHT / 2.0
                },
                radius: HOLE_RADIUS
            }.not());
        holes.push(
            Circle {
                center: Point {
                    x: HOLE_OFFSET + HOLE_SPACING * i as f32,
                    y: MAIN_HEIGHT / 2.0
                },
                radius: HOLE_RADIUS
            }.not());
    }
    holes
}

fn main() {
    let front_outline = front_outline().smooth(0.2, 0.01);
    let outline_stitch = front_outline.clone().shrink(STITCH_OFFSET);

    let mut targets: Vec<SyncBox> = holes().into_iter().map(|a| a.boxed()).collect();
    targets.push(front_outline.clone().boxed());

    let front_collar = AndThese { targets: targets };

    helper::display(vec![
        (front_collar.scale(100.0, 100.0).translate(50.0, 350.0).boxed(),            helper::Display::Lines),
        (front_outline.scale(100.0, 100.0).translate(50.0, 50.0).boxed(), helper::Display::Lines),
    ]);
}
