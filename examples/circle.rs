extern crate lux;
extern crate implicit;

mod helper;

use implicit::*;
use implicit::geom::*;


fn main() {
    let c1 = Circle {
        center: Point{x: 50.0, y: 50.0},
        radius: 50.0
    };

    let c2 = Circle {
        center: Point{x: 150.0, y: 50.0},
        radius: 50.0
    };

    let circle = c1.or(c2);

    helper::display(5.0, vec![
        (circle.boxed(), helper::Display::Dots),
        (circle.translate(0.0, 100.0).boxed(), helper::Display::Pixels),
        (circle.translate(0.0, 200.0).boxed(), helper::Display::Lines),
    ]);
}
