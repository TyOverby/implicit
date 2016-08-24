extern crate implicit;

use implicit::*;
use implicit::geom::*;

pub fn xored_circles() -> Xor<Xor<Circle, Circle>, Circle> {
    let circle_1 = Circle {
        center: Point { x: 50.0, y: 50.0 },
        radius: 50.0
    };

    let circle_2 = Circle {
        center: Point { x:  100.0, y: 50.0 },
        radius: 50.0
    };

    let circle_3 = Circle {
        center: Point { x: 75.0, y: 100.0 },
        radius: 50.0
    };

    let xored = Xor {
        left: circle_1,
        right: circle_2
    };

    Xor {
        left: xored,
        right: circle_3
    }
}

pub fn rect() -> Transformation<Polygon> {
    let poly = Polygon::new(vec![
                       Point { x:  50.0, y:  50.0 },
                       Point { x:  50.0, y: 200.0 },
                       Point { x: 200.0, y: 200.0 },
                       Point { x: 200.0, y:  50.0 },
                       ].into_iter());

    let mut transform = poly.transform();

    transform.matrix.translate(150.0, 150.0)
                    .rotate(3.14 / 8.0)
                    .translate(-150.0, -150.0);
    transform
}

pub struct Stripes;
impl Implicit for Stripes {
    fn sample(&self, point: Point) -> f32 {
        point.x.sin() * point.y.sin()
    }

    fn bounding_box(&self) -> Option<Rect> { None }
    fn follows_rules(&self) -> bool { false }
}

pub fn stripes() -> And<Stripes, Circle> {
    let lone = Circle {
        center: Point { x: 125.0, y: 200.0 },
        radius: 50.0
    };

    And { left: Stripes, right: lone }
}

pub fn poly() -> Transformation<Xor<Boundary<BoxCache<Polygon>>, Boundary<BoxCache<Polygon>>>> {
    let poly = Polygon::new(vec![
                       Point { x: 50.0, y: 50.0 },
                       Point { x: 200.0, y: 200.0 },
                       Point {x: 50.0, y: 200.0 },
                       ].into_iter());
    let poly = BoxCache::new(poly);

    let poly_outer = Boundary {
        target: poly.clone(),
        move_by: 10.0
    };

    let poly_inner = Boundary {
        target: poly,
        move_by: 50.0
    };

    let poly = Xor {
        left: poly_outer,
        right: poly_inner
    };

    let mut transform = poly.transform();

    transform.matrix.translate(50.0, 50.0)
                    .rotate(0.15)
                    .scale(1.25, 0.75);
    transform
}

pub fn front_collar() -> SyncBox {
    let neck_circ = 14.5;
    let front_len = (3.0 / 4.0) * neck_circ;
    let main_height = 1.0;
    let tri_offset = 0.5;

    // holes
    let hole_radius = 0.125;
    let hole_spacing = 0.25 + hole_radius * 2.0;
    let hole_offset = 0.5;

    let main_front_rect = Rectangle::new(Rect::from_points(
            &Point {x: 0.0, y: 0.0},
            &Point {x: front_len, y: main_height}));

    let left_triangle = Polygon::new(vec![
            Point { x: -tri_offset, y: main_height / 2.0 },
            Point { x: 0.0, y: 0.0 },
            Point { x: 0.0, y: main_height }].into_iter());

    let right_triangle = Polygon::new(vec![
            Point { x: front_len + tri_offset, y: main_height / 2.0 },
            Point { x: front_len, y: 0.0 },
            Point { x: front_len, y: main_height }].into_iter());

    let front_collar = main_front_rect.or(left_triangle).or(right_triangle);

    let mut holes = vec![];
    for i in 0 .. 4 {
        holes.push(
                Circle {
                    center: Point { x: hole_offset + hole_spacing * i as f32, y: main_height / 2.0 },
                    radius: hole_radius
                }.not().boxed());
    }
    let mut targets = holes;
    targets.push(front_collar.boxed());
    let front_collar = AndThese { targets: targets };

    front_collar.boxed()
}

pub fn rice_wall() -> SyncBox {
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
                            .or(wire_1)
                            .or(wire_2)
                            .or(wire_3)
                            .or(wire_4)
                            .or(center);

    finished.grow(0.00).boxed()
}

fn main() {
    panic!("not actually an example");
}
