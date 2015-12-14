extern crate lux;
extern crate implicit;

mod helper;

use implicit::*;

fn main() {
    let w = 2.0;
    let h = w * 1.61803398875;
    let wire_size = 0.1;

    let a = 1.0 / 4.0;
    let b = 1.0 / 3.0;

    let main_rect = Rectangle::new(Rect::from_points(
            &Point {x: 0.0, y: 0.0},
            &Point {x: w, y: h}));

    let cutout = main_rect.clone().shrink(wire_size);

    let wire_1 = Rectangle::new(Rect::from_point_and_size(
            &Point {x: 0.0, y: 0.0},
            &Vector {x: a * w, y: b * h }));

    let wire_2 = Rectangle::new(Rect::from_point_and_size(
            &Point {x: w - a * w, y: 0.0},
            &Vector {x: a * w, y: b * h }));

    let wire_3 = Rectangle::new(Rect::from_point_and_size(
            &Point {x: w - a * w, y: h - b * h},
            &Vector {x: a * w, y: b * h }));

    let wire_4 = Rectangle::new(Rect::from_point_and_size(
            &Point {x: 0.0, y: h - b * h},
            &Vector {x: a * w, y: b * h }));

    let center = Rectangle::new(Rect::from_point_and_size(
            &Point { x: b * w, y: b * b * h },
            &Vector { x: b * w, y: h - h * b * b}));


    let finished = main_rect.and_not(cutout)
                            .or(wire_1.outline(wire_size))
                            .or(wire_2.outline(wire_size))
                            .or(wire_3.outline(wire_size))
                            .or(wire_4.outline(wire_size))
                            .or(center.outline(wire_size));

    let f = GenericShape::Boxed(Box::new(finished.grow(0.00)));
    let f = f.scale(100.0, 100.0);

    helper::display(f, vec![helper::Display::Lines]);
}
