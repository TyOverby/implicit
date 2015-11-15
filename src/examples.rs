use implicit::*;

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
    let mut transform = Transformation {
        target: poly,
        matrix: Matrix::new(),
    };

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

    let mut transform = Transformation {
        target: poly,
        matrix: Matrix::new()
    };

    transform.matrix.translate(50.0, 50.0)
                    .rotate(0.15)
                    .scale(1.25, 0.75);
    transform
}
