#![feature(test)]

extern crate implicit;
extern crate flame;
extern crate test;
extern crate rayon;

use implicit::*;
use implicit::geom::*;
use std::fs::File;

const ITERS: usize = 1;

/// Why do SCALE and RESOLUTION not cancel eachother out when increased at the same rate?
const RESOLUTION: f32 = 1.0;
const SCALE: f32 = RESOLUTION;

fn main() {
    let r_conf = rayon::Configuration::new();
    rayon::initialize(r_conf);

    let circle = Circle { center: Point{x: 0.0, y: 0.0}, radius: 500.0};
    let square = Rectangle::new(Rect::from_point_and_size(&Point { x: 0.0, y: 0.0 }, &Vector { x: 250.0, y: 250.0 }));
    let poly = ::flame::span_of("prep", || circle.or(square).smooth(10.0, RESOLUTION));
    println!("{} lines", poly.target.polys[0].lines().len());
    let poly = poly.scale(SCALE, SCALE);

    let mut total = 0.0;
    let mut minimum = ::std::f32::INFINITY;
    let mut maximum = -::std::f32::INFINITY;
    let mut min_index = 0;
    for i in 0 .. ITERS {
        flame::start("real deal");
        let a = render(poly.clone(), &RenderMode::Outline, RESOLUTION, true);
        test::black_box(a);
        let end = flame::end("real deal");
        let end = end as f32 / 1000000.0;
        total += end;
        minimum = minimum.min(end);
        maximum = maximum.max(end);
        min_index = i;
    }

    ::flame::dump_html(&mut File::create("./flamegraph.html").unwrap());
    let threads = ::flame::threads();
    println!("{}", ::flame::Thread::into_json_list(&threads));
    //println!("avg: {}, min: {}, max: {}", total / ITERS as f32, minimum, maximum);
}
