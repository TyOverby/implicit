#![feature(test)]

extern crate implicit;
extern crate lux;
extern crate flame;
extern crate test;
extern crate rayon;

mod helper;

use implicit::*;
use implicit::geom::*;
use std::fs::File;

const ITERS: usize = 1;

/// Why do SCALE and RESOLUTION not cancel eachother out when increased at the same rate?
const RECURSION_DEPTH: u32 = 8;
const SCALE: f32 = 1.0;

fn main() {
    let circle = Circle { center: Point{x: 0.0, y: 0.0}, radius: 500.0};
    let square = Rectangle::new(Rect::from_point_and_size(&Point { x: 0.0, y: 0.0 }, &Vector { x: 500.0, y: 500.0 }));
    let poly = ::flame::span_of("prep", || circle.or(square.clone()).smooth(10.0, RECURSION_DEPTH));
    println!("{} lines", poly.target.polys[0].lines().len());
    let poly = poly.scale(SCALE);

    let mut total = 0.0;
    let mut minimum = ::std::f32::INFINITY;
    let mut maximum = -::std::f32::INFINITY;
    let mut _min_index = 0;
    for i in 0 .. ITERS {
        flame::start("real deal");
        let a = render(poly.clone(), &RenderMode::Outline, RECURSION_DEPTH, true);
        test::black_box(a);
        let end = flame::end("real deal");
        let end = end as f32 / 1000000.0;
        total += end;
        minimum = minimum.min(end);
        maximum = maximum.max(end);
        _min_index = i;
    }

    ::flame::dump_html(&mut File::create("./flamegraph.html").unwrap()).unwrap();
    let threads = ::flame::threads();
//    println!("{}", ::flame::Thread::into_json_list(&threads));
    println!("avg: {}, min: {}, max: {}", total / ITERS as f32, minimum, maximum);

    //helper::display(&[&poly]);
}
