#![feature(test)]

extern crate implicit;
extern crate flame;
extern crate test;

use implicit::*;
use implicit::geom::*;

const ITERS: u32 = 100;
const RESOLUTION: f32 = 1.0;

fn main() {
    let circle = Circle { center: Point{x: 0.0, y: 0.0}, radius: 100.0};
    let square = Rectangle::new(Rect::from_point_and_size(&Point { x: 0.0, y: 0.0 }, &Vector { x: 50.0, y: 50.0 }));
    let poly = ::flame::span_of("prep", || circle.or(square).smooth(10.0, RESOLUTION));

    let mut total = 0.0;
    let mut minimum = ::std::f32::INFINITY;
    let mut maximum = -::std::f32::INFINITY;
    for _ in 0 .. ITERS {
        flame::start("real deal");
        let a = render(poly.clone(), &RenderMode::Outline, RESOLUTION, true);
        test::black_box(a);
        let end = flame::end("real deal");
        let end = end as f32 / 1000000.0;
        total += end;
        minimum = minimum.min(end);
        maximum = maximum.max(end);
    }

    println!("avg: {}, min: {}, max: {}", total / ITERS as f32, minimum, maximum);
}
