#![allow(dead_code)]

extern crate vecmath;

mod geom;
mod marching_squares;
mod quadtree;
mod line_join;
pub use geom::*;
pub use quadtree::*;
pub use marching_squares::*;
pub use line_join::*;


// TODO: this should be unsized
pub trait Implicit {
    /// Returns the distance from a point to the nearest edge of a surface.
    ///
    /// If the point is outside of the surface, return a positive number.
    /// If the point is inside of the surface, return a negative number.
    /// If the point is on the line, return 0.
    fn sample(&self, pos: Point) -> f32;

    /// Returns a bounding box that surrounds a shape (if one exists).
    ///
    /// If the shape is infinite, return None.
    fn bounding_box(&self) -> Option<Rect>;
}

pub enum GenericShape {
    Circle(Circle),
    Polygon(Polygon),
    And(And<Box<GenericShape>, Box<GenericShape>>),
    Or(Or<Box<GenericShape>, Box<GenericShape>>),
    Xor(Xor<Box<GenericShape>, Box<GenericShape>>),
    Boundary(Boundary<Box<GenericShape>>),
    Not(Not<Box<GenericShape>>),
    BoxCache(BoxCache<Box<GenericShape>>),
    Transformation(Transformation<Box<GenericShape>>),
}

#[derive(Copy, Clone)]
pub struct Circle {
    pub center: Point,
    pub radius: f32
}

#[derive(Clone)]
pub struct Rectangle {
    rect: Rect,
    poly: Polygon
}

#[derive(Copy, Clone)]
pub struct And<A: Implicit, B: Implicit> {
    pub left: A,
    pub right: B,
}

#[derive(Copy, Clone)]
pub struct Or<A: Implicit, B: Implicit> {
    pub left: A,
    pub right: B,
}

#[derive(Copy, Clone)]
pub struct Xor<A: Implicit, B: Implicit> {
    pub left: A,
    pub right: B,
}

#[derive(Copy, Clone)]
pub struct Boundary<A: Implicit> {
    pub target: A,
    pub move_by: f32
}

#[derive(Copy, Clone)]
pub struct Not<A: Implicit> {
    pub target: A,
}

#[derive(Copy, Clone)]
pub struct BoxCache<A: Implicit> {
    target: A,
    cache: Option<Rect>
}

#[derive(Copy, Clone)]
pub struct Transformation<A: Implicit> {
    pub target: A,
    pub matrix: Matrix,
}


impl <A: Implicit> BoxCache<A> {
    pub fn new(target: A) -> BoxCache<A> {
        let bb = target.bounding_box();
        BoxCache {
            target: target,
            cache: bb
        }
    }
}

impl <A: Implicit> Implicit for Transformation<A> {
    fn sample(&self, pos: Point) -> f32 {
        self.target.sample(self.matrix.transform_point_inv(&pos))
    }

    fn bounding_box(&self) -> Option<Rect> {
        let bb = match self.target.bounding_box() {
            Some(bb) => bb,
            None => return None
        };

        let a = self.matrix.transform_point(&bb.top_left());
        let b = self.matrix.transform_point(&bb.top_right());
        let c = self.matrix.transform_point(&bb.bottom_left());
        let d = self.matrix.transform_point(&bb.bottom_right());

        let mut rect = Rect::null_at(&a);
        rect.expand_to_include(&b);
        rect.expand_to_include(&c);
        rect.expand_to_include(&d);

        Some(rect)
    }
}

impl <A: Implicit> Implicit for Box<A> {
    fn sample(&self, pos: Point) -> f32 {
        (**self).sample(pos)
    }

    fn bounding_box(&self) -> Option<Rect> {
        (**self).bounding_box()
    }
}

impl Implicit for GenericShape {
    fn sample(&self, pos: Point) -> f32 {
        match self {
            &GenericShape::Circle(ref circle) => circle.sample(pos),
            &GenericShape::Polygon(ref poly) => poly.sample(pos),
            &GenericShape::And(ref and) => and.sample(pos),
            &GenericShape::Or(ref or) => or.sample(pos),
            &GenericShape::Xor(ref xor) => xor.sample(pos),
            &GenericShape::Boundary(ref b) => b.sample(pos),
            &GenericShape::Not(ref n) => n.sample(pos),
            &GenericShape::BoxCache(ref c) => c.sample(pos),
            &GenericShape::Transformation(ref t) => t.sample(pos),
        }
    }

    fn bounding_box(&self) -> Option<Rect> {
        match self {
            &GenericShape::Circle(ref circle) => circle.bounding_box(),
            &GenericShape::Polygon(ref poly) => poly.bounding_box(),
            &GenericShape::And(ref and) => and.bounding_box(),
            &GenericShape::Or(ref or) => or.bounding_box(),
            &GenericShape::Xor(ref xor) => xor.bounding_box(),
            &GenericShape::Boundary(ref b) => b.bounding_box(),
            &GenericShape::Not(ref n) => n.bounding_box(),
            &GenericShape::BoxCache(ref c) => c.bounding_box(),
            &GenericShape::Transformation(ref t) => t.bounding_box(),
        }
    }
}

impl <I: Implicit> Implicit for BoxCache<I> {
    fn sample(&self, pos: Point) -> f32 {
        self.target.sample(pos)
    }
    fn bounding_box(&self) -> Option<Rect> {
        self.cache
    }
}

impl <I: Implicit> Implicit for Not<I> {
    fn sample(&self, pos: Point) -> f32 {
        -self.target.sample(pos)
    }
    fn bounding_box(&self) -> Option<Rect> {
        None
    }
}

impl Implicit for Polygon {
    fn sample(&self, pos: Point) -> f32 {
        let mut min = ::std::f32::INFINITY;
        for line in self.lines() {
            min = min.min(line.dist_to_point(&pos));
        }

        let ray = Ray(pos, Vector{x: 1.0, y: 0.0});
        let mut num_hits = 0;
        for line in self.lines() {
            if ray.intersect_with_line(line).is_some() {
                num_hits += 1;
            }
        }

        if num_hits % 2 == 0 {
            min
        } else {
            -min
        }
    }

    fn bounding_box(&self) -> Option<Rect> {
        let mut min_x = ::std::f32::INFINITY;
        let mut min_y = ::std::f32::INFINITY;
        let mut max_x = -::std::f32::INFINITY;
        let mut max_y = -::std::f32::INFINITY;

        for line in self.lines().iter() {
            min_x = min_x.min(line.0.x).min(line.1.x);
            min_y = min_y.min(line.0.y).min(line.1.y);
            max_x = max_x.max(line.0.x).max(line.1.x);
            max_y = max_y.max(line.0.y).max(line.1.y);
        }

        Some(Rect::from_points(&Point{x: min_x, y: min_y}, &Point{x: max_x, y: max_y}))
    }
}

impl Implicit for Circle {
    fn sample(&self, pos: Point) -> f32 {
        let Point{x: a, y: b} = pos;
        let Point{x: c, y: d} = self.center;
        let dx = a - c;
        let dy = b - d;
        let dist = (dx * dx + dy * dy).sqrt();
        dist - self.radius
    }

    fn bounding_box(&self) -> Option<Rect> {
        let Point{x: cx, y: cy} = self.center;
        let r = self.radius;
        Some(Rect {
            top_left: Point {
                x: cx - r,
                y: cy - r
            },
            bottom_right: Point{
                x: cx + r,
                y: cy + r
            }
        })
    }
}

impl <A: Implicit, B: Implicit> Implicit for And<A, B> {
    fn sample(&self, pos: Point) -> f32 {
        let left_sample = self.left.sample(pos);
        let right_sample = self.right.sample(pos);
        if left_sample > right_sample {
            left_sample
        } else {
            right_sample
        }
    }

    fn bounding_box(&self) -> Option<Rect> {
        let left_bb = self.left.bounding_box();
        let right_bb = self.right.bounding_box();

        match (left_bb, right_bb) {
            (Some(left_bb), Some(right_bb)) => Some(left_bb.intersect_with(&right_bb)),
            (Some(left_bb), None) => Some(left_bb),
            (None, Some(right_bb)) => Some(right_bb),
            (None, None) => None
        }
    }
}

impl <A: Implicit, B: Implicit> Implicit for Or<A, B> {
    fn sample(&self, pos: Point) -> f32 {
        let left_sample = self.left.sample(pos);
        let right_sample = self.right.sample(pos);
        if left_sample < right_sample {
            left_sample
        } else {
            right_sample
        }
    }

    fn bounding_box(&self) -> Option<Rect> {
        let left_bb = self.left.bounding_box();
        let right_bb = self.right.bounding_box();
        match (left_bb, right_bb) {
            (Some(left_bb), Some(right_bb)) => Some(left_bb.union_with(&right_bb)),
            (_, _) => None
        }
    }
}

impl <A: Implicit, B: Implicit> Implicit for Xor<A, B> {
    fn sample(&self, pos: Point) -> f32 {
        let left_sample = self.left.sample(pos);
        let right_sample = self.right.sample(pos);

        // Both are contained
        if left_sample < 0.0 && right_sample < 0.0 {
            (-left_sample).min(-right_sample)
        // Neither are contained
        } else if left_sample > 0.0 && right_sample > 0.0 {
            (left_sample).min(right_sample)
        }
        // Contained on the left, not on the right
        else if left_sample < right_sample {
            if -left_sample > right_sample {
                -right_sample
            } else {
                left_sample
            }
        // Contained on the right, not on the left
        } else {
            if -right_sample > left_sample {
                -left_sample
            } else {
                right_sample
            }
        }
    }

    fn bounding_box(&self) -> Option<Rect> {
        let left_bb = self.left.bounding_box();
        let right_bb = self.right.bounding_box();
        match (left_bb, right_bb) {
            (Some(left_bb), Some(right_bb)) => Some(left_bb.union_with(&right_bb)),
            (_, _) => None
        }
    }
}

impl <A: Implicit> Implicit for Boundary<A> {
    fn sample(&self, pos: Point) -> f32 {
        self.target.sample(pos) - self.move_by
    }

    fn bounding_box(&self) -> Option<Rect> {
        self.target.bounding_box().map(|r| r.expand(self.move_by, self.move_by, self.move_by, self.move_by))
    }
}

impl Rectangle {
    fn recompute_poly(&mut self) {
        let v = vec![self.rect.top_left(),
                     self.rect.top_right(),
                     self.rect.bottom_right(),
                     self.rect.bottom_left()];
        self.poly = Polygon::new(v.into_iter());
    }
    pub fn new(rect: Rect) -> Rectangle {
        let mut r = Rectangle {
            rect: rect,
            poly: Polygon::new(vec![].into_iter())
        };
        r.recompute_poly();
        r
    }

    pub fn with_underlying<R, F: FnOnce(&mut Rect) -> R>(&mut self, f: F) -> R {
        let r = f(&mut self.rect);
        self.recompute_poly();
        r
    }
}
