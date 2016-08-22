extern crate lux;
extern crate implicit;

mod helper;

use implicit::*;
use implicit::geom::*;

fn main() {
    let scale = 1.0;

    let poly = vec![Point{x: 2.0 * scale, y: 1.0 * scale},
                    Point{x: 2.5 * scale, y: 1.0 * scale},
                    Point{x: 4.0 * scale, y: 5.0 * scale},
                    Point{x: 1.0 * scale, y: 5.0 * scale}];
    let f = Polygon::new(poly.into_iter());

    let f = f.scale(100.0);

    helper::display(&[&f]);
}
