extern crate lux;
extern crate implicit;

mod helper;

use implicit::*;
use implicit::geom::*;

fn main() {
    let wire_size = 0.7;

    let poly = vec![Point{x: 2.0, y: 1.0},
                    Point{x: 2.5, y: 1.0},
                    Point{x: 4.0, y: 5.0},
                    Point{x: 1.0, y: 5.0}];
    let f = Polygon::new(poly.into_iter());
    //let f = f.outline(wire_size);

    let f = f.scale(100.0, 100.0);

    helper::display(5.0, vec![(f.boxed(), helper::Display::Dots)]);
}
