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
    pub fn null() -> Rect {
        Rect {
            top_left: Point { x: 0.0, y: 0.0 },
            bottom_right: Point { x: 0.0, y: 0.0}
        }
    }

    pub fn null_at(point: Point) -> Rect {
        Rect {
            top_left: point,
            bottom_right: point,
        }
    }

    pub fn expanded_by(&self, point: Point) -> Rect {
        let mut r = self.clone();
        r.expand_to_include(point);
        r
    }

    pub fn expand_to_include(&mut self, point: Point) {
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

    pub fn merged_with(&self, other: &Rect) -> Rect {
        let mut r = self.clone();
        r.expand_to_include(other.top_left);
        r.expand_to_include(other.bottom_right);
        r
    }

    pub fn contains(&self, p: &Point) -> bool {
        p.x >= self.top_left.x &&
        p.x <= self.bottom_right.x &&
        p.y >= self.top_left.y &&
        p.y <= self.bottom_right.y
    }
}
