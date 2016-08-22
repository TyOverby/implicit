extern crate lux;
extern crate implicit;

mod helper;

use implicit::{Implicit, Circle};
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

    helper::display(&[&circle]);
}
