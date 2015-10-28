use std::ops::{Neg, Add};

#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub struct Vector {
    pub x: f32,
    pub y: f32
}

#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub struct Line(pub Point, pub Point);

#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub struct Rect
{
    pub top_left: Point,
    pub bottom_right: Point
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
        Rect {
            top_left: Point { x: 0.0, y: 0.0 },
            bottom_right: Point { x: 0.0, y: 0.0 }
        }
    }

    pub fn null_at(point: &Point) -> Rect {
        Rect {
            top_left: *point,
            bottom_right: *point,
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
        other.contains(&self.top_left) ||
        other.contains(&self.bottom_right) ||
        self.contains(&other.top_left) ||
        self.contains(&other.bottom_right)
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

    pub fn width(&self) -> f32 {
        self.bottom_right.x - self.top_left.x
    }

    pub fn height(&self) -> f32 {
        self.bottom_right.y - self.top_left.y
    }

    pub fn split_quad(&self) -> [Rect; 4] {
        let half = Vector { x: self.width(), y: self.height() };
        [
            // x _
            // _ _
            Rect::from_point_and_size(
                &self.top_left,
                &half),
            // _ x
            // _ _
            Rect::from_point_and_size(
                &Point { x: self.top_left.x + half.x, y: self.top_left.y },
                &half),
            // _ _
            // x _
            Rect::from_point_and_size(
                &Point { x: self.top_left.x, y: self.top_left.y + half.y },
                &half),
            // _ _
            // _ x
            Rect::from_point_and_size(
                &(self.top_left + half),
                &half)
        ]
    }
}
