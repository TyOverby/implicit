use std::ops::{Neg, Add, Sub, Mul, Div};

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub struct Vector {
    pub x: f32,
    pub y: f32
}

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub struct Line(pub Point, pub Point);

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub struct Rect
{
    pub top_left: Point,
    pub bottom_right: Point
}

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub struct Ray(pub Point, pub Vector);

#[derive(PartialOrd, PartialEq, Clone, Debug)]
pub struct Polygon {
    points: Vec<Point>,
    lines: Vec<Line>,
}

impl Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Vector {
        Vector {
            x: -self.x,
            y: -self.y
        }
    }
}

impl Sub<Vector> for Point {
    type Output = Point;
    fn sub(self, rhs: Vector) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

impl Sub<Point> for Vector {
    type Output = Point;
    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

impl Add<Vector> for Point {
    type Output = Point;
    fn add(self, rhs: Vector) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl Add<Point> for Vector {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl Sub<Point> for Point {
    type Output = Vector;
    fn sub(self, rhs: Point) -> Vector {
        Vector {
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

impl Add<Vector> for Vector {
    type Output = Vector;
    fn add(self, rhs: Vector) -> Vector {
        Vector {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl Mul<f32> for Vector {
    type Output = Vector;
    fn mul(self, rhs: f32) -> Vector {
        Vector {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<f32> for Vector {
    type Output = Vector;
    fn div(self, rhs: f32) -> Vector {
        Vector {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}


impl Line {
    pub fn dist_to_point(&self, p: &Point) -> f32 {
        #[inline(always)]
        fn sqr(x: f32) -> f32 { x * x }
        #[inline(always)]
        fn dist2(v: &Point, w: &Point) -> f32 {
            sqr(v.x - w.x) + sqr(v.y - w.y)
        }
        #[inline(always)]
        fn dist_to_segment_squared(p: &Point, v: &Point, w: &Point) -> f32 {
            let l2 = dist2(v, w);
            if l2 == 0.0 { //  TODO: epsilon
                return dist2(p, v);
            }
            let t = ((p.x - v.x) * (w.x - v.x) + (p.y - v.y) * (w.y - v.y)) / l2;
            if t < 0.0 {
                dist2(p, v)
            } else if t > 1.0 {
                dist2(p, w)
            } else {
                dist2(p, &Point {
                    x: v.x + t * (w.x - v.x),
                    y: v.y + t * (w.y - v.y)
                })
            }
        }

        dist_to_segment_squared(p, &self.0, &self.1).sqrt()
    }
}

impl Rect {
    pub fn from_points(p1: &Point, p2: &Point) -> Rect {
        let mut r = Rect::null_at(&p1);
        r.expand_to_include(&p2);
        r
    }

    pub fn from_point_and_size(point: &Point, size: &Vector) -> Rect {
        assert!(size.x > 0.0);
        assert!(size.y > 0.0);
        Rect {
            top_left: *point,
            bottom_right: *point + *size
        }
    }

    pub fn null() -> Rect {
        let nan = ::std::f32::NAN;
        Rect {
            top_left: Point {x: nan, y: nan},
            bottom_right: Point {x: nan, y: nan}
        }
    }

    pub fn null_at(point: &Point) -> Rect {
        Rect {
            top_left: *point,
            bottom_right: *point,
        }
    }

    pub fn expand(&self, left: f32, top: f32, right: f32, bottom: f32) -> Rect {
        let top_left_vec = Vector { x: left, y: top };
        let bottom_right_vec = Vector { x: right, y: bottom };
        Rect {
            top_left: self.top_left - top_left_vec,
            bottom_right: self.bottom_right + bottom_right_vec,
        }
    }

    pub fn width(&self) -> f32 {
        self.bottom_right.x - self.top_left.x
    }

    pub fn height(&self) -> f32 {
        self.bottom_right.y - self.top_left.y
    }

    fn left(&self) -> f32 {
        self.top_left.x
    }

    fn right(&self) -> f32 {
        self.bottom_right.x
    }

    fn top(&self) -> f32 {
        self.top_left.y
    }

    fn bottom(&self) -> f32 {
        self.bottom_right.y
    }

    pub fn top_left(&self) -> Point {
        self.top_left
    }

    pub fn bottom_right(&self) -> Point {
        self.bottom_right
    }

    pub fn bottom_left(&self) -> Point {
        Point {
            x: self.top_left().x,
            y: self.bottom_right().y
        }
    }

    pub fn top_right(&self) -> Point {
        Point {
            x: self.bottom_right().x,
            y: self.top_left().y
        }
    }


    pub fn expanded_by(&self, point: &Point) -> Rect {
        let mut r = self.clone();
        r.expand_to_include(point);
        r
    }

    pub fn expand_to_include(&mut self, point: &Point) {
        if point.x < self.top_left.x {
            self.top_left.x = point.x;
        }
        if point.y < self.top_left.y {
            self.top_left.y = point.y;
        }

        if point.x > self.bottom_right.x {
            self.bottom_right.x = point.x;
        }
        if point.y > self.bottom_right.y {
            self.bottom_right.y = point.y;
        }
    }

    pub fn union_with(&self, other: &Rect) -> Rect {
        let mut r = self.clone();
        r.expand_to_include(&other.top_left);
        r.expand_to_include(&other.bottom_right);
        r
    }

    pub fn contains(&self, p: &Point) -> bool {
        p.x > self.top_left.x &&
        p.x < self.bottom_right.x &&
        p.y > self.top_left.y &&
        p.y < self.bottom_right.y
    }

    pub fn does_intersect(&self, other: &Rect) -> bool{
        let r1 = self;
        let r2 = other;

        // From stack overflow:
        // http://gamedev.stackexchange.com/a/913
        !( r2.left() > r1.right()
        || r2.right() < r1.left()
        || r2.top() > r1.bottom()
        || r2.bottom() < r1.top())
    }

    pub fn intersect_with(&self, other: &Rect) -> Rect {
        let mut r = Rect::null();
        let mut added = 0;
        if self.contains(&other.top_left) {
            r.expand_to_include(&other.top_left);
            added += 1;
        }
        if self.contains(&other.bottom_right) {
            r.expand_to_include(&other.bottom_right);
            added += 1;
        }

        // Bail early if we've already found the intersection
        if added == 2 {
            return r;
        }

        if other.contains(&self.top_left) {
            r.expand_to_include(&self.top_left);
        }

        // Bail early if we've already found the intersection
        if added == 2 {
            return r;
        }

        if other.contains(&self.bottom_right) {
            r.expand_to_include(&self.bottom_right);
        }
        r
    }

    pub fn midpoint(&self) -> Point {
        let half = Vector { x: self.width() / 2.0, y: self.height() / 2.0 };
        self.top_left() + half
    }

    pub fn split_quad(&self) -> [Rect; 4] {
        let half = Vector { x: self.width() / 2.0, y: self.height() / 2.0 };
        [
            // x _
            // _ _
            Rect::from_point_and_size(
                &self.top_left,
                &half),
            // _ x
            // _ _
            Rect::from_point_and_size(
                &Point { x: self.top_left.x + half.x, .. self.top_left},
                &half),
            // _ _
            // x _
            Rect::from_point_and_size(
                &Point { y: self.top_left.y + half.y, .. self.top_left },
                &half),
            // _ _
            // _ x
            Rect::from_point_and_size(
                &(self.top_left + half),
                &half)
        ]
    }
}

impl Polygon {
    pub fn new<I: Iterator<Item=Point>>(i: I) -> Polygon {
        let points: Vec<_> = i.collect();
        let lines  = Polygon::compute_lines(&points[..]);
        Polygon {
            points: points,
            lines: lines,
        }
    }

    // TODO: make this a lazy iterator.
    fn compute_lines(from: &[Point]) -> Vec<Line> {
        let mut out = vec![];
        for window in from.windows(2) {
            out.push(Line(window[0], window[1]));
        }
        if from.len() > 2 {
            out.push(Line(*from.first().unwrap(), *from.last().unwrap()));
        }
        out
    }

    pub fn lines(&self) -> &[Line] {
        &self.lines
    }
}

impl Vector {
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalized(&self) -> Vector {
        let m = self.magnitude();
        Vector {
            x: self.x / m,
            y: self.y / m,
        }
    }

    pub fn cross(&self, other: &Vector) -> f32 {
        self.x * other.y - self.y * other.x
    }

    pub fn dot(&self, other: &Vector) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

impl Ray {
    pub fn intersect_with_line(&self, line: &Line) -> Option<Point> {
        let ray_origin = self.0;
        let ray_direction = self.1;
        let point_1 = line.0;
        let point_2 = line.1;

        let v1 = ray_origin - point_1;
        let v2 = point_2 - point_1;
        let v3 = Vector {x: -ray_direction.y, y: ray_direction.x};

        let t1 = v2.cross(&v1) / v2.dot(&v3);
        let t2 = v1.dot(&v3) / v2.dot(&v3);

        if t1 >= 0.0 && t2 >= 0.0 && t2 <= 1.0 {
            let normalized_direction = ray_direction.normalized();
            Some(ray_origin + normalized_direction * t1)
        } else {
            None
        }
    }
}
