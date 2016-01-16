#![allow(dead_code)]

extern crate vecmath;
extern crate rand;
extern crate itertools;
extern crate crossbeam;
extern crate flame;

mod private;

pub use private::render::{render, sampling_points, RenderMode, OutputMode};
pub use private::geom;
pub use private::quadtree::QuadTree;

use private::{Rect, Point, Polygon, Matrix, Vector, Line, Ray};
use std::sync::Arc;

// TODO: this should be unsized
pub trait Implicit: Sync + Send {
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

    fn boxed(self) -> Box<Implicit> where Self: Sized + 'static {
        Box::new(self)
    }

    /// True if the shape follows all the rules about implicit shapes.
    fn follows_rules(&self) -> bool;

    fn and<B: Implicit>(self, other: B) -> And<Self, B> where Self: Sized {
        And {
            left: self,
            right: other
        }
    }

    fn and_not<B: Implicit>(self, other: B) -> And<Self, Not<B>> where Self: Sized {
        And {
            left: self,
            right: other.not(),
        }
    }

    fn or<B: Implicit>(self, other: B) -> Or<Self, B> where Self: Sized {
        Or {
            left: self,
            right: other
        }
    }

    fn xor<B: Implicit>(self, other: B) -> Xor<Self, B> where Self: Sized {
        Xor {
            left: self,
            right: other
        }
    }

    fn shrink(self, by: f32) -> Boundary<Self> where Self: Sized {
        let by = by.max(0.0);
        Boundary {
            target: self,
            move_by: -by
        }
    }

    fn grow(self, by: f32) -> Boundary<Self> where Self: Sized {
        let by = by.max(0.0);
        Boundary {
            target: self,
            move_by: by
        }
    }

    fn cache_bounding_box(self) -> BoxCache<Self> where Self: Sized {
        BoxCache::new(self)
    }

    fn transform(self) -> Transformation<Self> where Self: Sized {
        Transformation::new(self)
    }

    fn scale(self, sx: f32, sy: f32) -> Transformation<Self> where Self: Sized {
        let mut r = Transformation::new(self);
        r.matrix.scale(sx, sy);
        r
    }

    fn translate(self, x: f32, y: f32) -> Transformation<Self> where Self: Sized {
        let mut r = Transformation::new(self);
        r.matrix.translate(x, y);
        r
    }

    fn rotate(self, rads: f32) -> Transformation<Self> where Self: Sized {
        let mut r = Transformation::new(self);
        r.matrix.rotate(rads);
        r
    }

    fn not(self) -> Not<Self> where Self: Sized {
        Not { target: self}
    }

    fn outline(self, distance: f32) -> And<Self, Not<Boundary<Self>>> where Self: Sized + Clone {
        self.clone().and(self.shrink(distance).not())
    }

    fn borrow(&self) -> &Self where Self: Sized {
        self
    }

    fn fix_rules(&self, resolution: f32) -> PolyGroup where Self: Sized {
        let rendered = render(self, RenderMode::Outline, resolution, true);
        let lines = if let OutputMode::Outline(lines) = rendered {
            lines
        } else {
            panic!("somehow didn't get outline.");
        };

        PolyGroup {
            polys: lines.into_iter().map(|p| Polygon::new(p.into_iter())).collect()
        }
    }
}



pub enum GenericShape<'a> {
    Circle(Circle),
    Polygon(Polygon),
    And(And<Box<GenericShape<'a>>, Box<GenericShape<'a>>>),
    Or(Or<Box<GenericShape<'a>>, Box<GenericShape<'a>>>),
    Xor(Xor<Box<GenericShape<'a>>, Box<GenericShape<'a>>>),
    Boundary(Boundary<Box<GenericShape<'a>>>),
    Not(Not<Box<GenericShape<'a>>>),
    BoxCache(BoxCache<Box<GenericShape<'a>>>),
    Boxed(Box<Implicit>),
    Transformation(Transformation<Box<GenericShape<'a>>>),
    Ref(&'a Implicit)
}

#[derive(Clone)]
pub struct PolyGroup {
    polys: Vec<Polygon>
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

#[derive(Clone)]
pub struct OrThese<A: Implicit> {
    pub targets: Vec<A>
}

#[derive(Clone)]
pub struct AndThese<A: Implicit> {
    pub targets: Vec<A>
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

impl Implicit for PolyGroup {
    fn sample(&self, pos: Point) -> f32 {
        const INF: f32 = ::std::f32::INFINITY;
        let sampled = self.polys.iter().map(|a| a.sample(pos)).collect::<Vec<_>>();
        let inside_count = sampled.iter().filter(|&&a| a < 0.0).count();
        let closest = sampled.iter().fold(INF, |best, contender| {
            let contender = contender.abs();
            if contender < best { contender } else { best }
        });

        if inside_count % 2 == 0 {
            closest
        } else {
            -closest
        }
    }

    fn bounding_box(&self) -> Option<Rect> {
        Some(self.polys.iter()
                       .map(|a| a.bounding_box().unwrap())
                       .fold(Rect::null(), |a, b| {
                           a.union_with(&b)
                        }))
    }

    fn follows_rules(&self) -> bool { true }
}

impl <A: Implicit> Transformation<A> {
    pub fn new(a: A) -> Transformation<A> {
        Transformation {
             target: a,
             matrix: Matrix::new()
        }
    }
}

impl <A: Implicit> OrThese<A> {
    pub fn new(targets: Vec<A>) -> OrThese<A> {
        OrThese { targets: targets }
    }
}

impl <A: Implicit> AndThese<A> {
    pub fn new(targets: Vec<A>) -> AndThese<A> {
        AndThese { targets: targets }
    }
}

impl <A: Implicit> Implicit for OrThese<A> {
    fn sample(&self, pos: Point) -> f32 {
        let mut minimum = std::f32::INFINITY;
        for p in &self.targets {
            minimum = minimum.min(p.sample(pos));
        }
        minimum
    }

    fn bounding_box(&self) -> Option<Rect> {
        let mut bb = Rect::null();
        for p in &self.targets {
            if let Some(p_bb) = p.bounding_box() {
                bb = bb.union_with(&p_bb);
            }
        }

        if bb.is_null() {
            None
        } else {
            Some(bb)
        }
    }

    fn follows_rules(&self) -> bool {
        self.targets.iter().all(|a| a.follows_rules())
    }
}

impl <A: Implicit> Implicit for AndThese<A> {
    fn sample(&self, pos: Point) -> f32 {
        let mut maximum = -std::f32::INFINITY;
        for p in &self.targets {
            maximum = maximum.max(p.sample(pos));
        }
        maximum
    }

    fn bounding_box(&self) -> Option<Rect> {
        let mut bb = Rect::null();
        for p in &self.targets {
            if let Some(p_bb) = p.bounding_box() {
                if bb.is_null() {
                    bb = p_bb;
                } else {
                    bb = bb.intersect_with(&p_bb);
                }
            }
        }

        if bb.is_null() {
            None
        } else {
            Some(bb)
        }
    }

    fn follows_rules(&self) -> bool {
        self.targets.iter().all(|a| a.follows_rules())
    }
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

    fn follows_rules(&self) -> bool {
        self.target.follows_rules()
    }
}

impl <'a> Implicit for &'a Implicit {
    fn sample(&self, pos: Point) -> f32 {
        (**self).sample(pos)
    }

    fn bounding_box(&self) -> Option<Rect> {
        (**self).bounding_box()
    }

    fn follows_rules(&self) -> bool {
        (**self).follows_rules()
    }
}

impl <A: Implicit + ?Sized> Implicit for Box<A> {
    fn sample(&self, pos: Point) -> f32 {
        (**self).sample(pos)
    }

    fn bounding_box(&self) -> Option<Rect> {
        (**self).bounding_box()
    }

    fn follows_rules(&self) -> bool {
        (**self).follows_rules()
    }
}

impl <A: Implicit + ?Sized + Sync + Send> Implicit for Arc<A> {
    fn sample(&self, pos: Point) -> f32 {
        (**self).sample(pos)
    }

    fn bounding_box(&self) -> Option<Rect> {
        (**self).bounding_box()
    }

    fn follows_rules(&self) -> bool {
        (**self).follows_rules()
    }
}

impl <'a> Implicit for GenericShape<'a> {
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
            &GenericShape::Boxed(ref t) => t.sample(pos),
            &GenericShape::Ref(ref t) => t.sample(pos),
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
            &GenericShape::Boxed(ref t) => t.bounding_box(),
            &GenericShape::Ref(ref t) => t.bounding_box(),
        }
    }

    fn follows_rules(&self) -> bool {
        match self {
            &GenericShape::Circle(ref circle) => circle.follows_rules(),
            &GenericShape::Polygon(ref poly) => poly.follows_rules(),
            &GenericShape::And(ref and) => and.follows_rules(),
            &GenericShape::Or(ref or) => or.follows_rules(),
            &GenericShape::Xor(ref xor) => xor.follows_rules(),
            &GenericShape::Boundary(ref b) => b.follows_rules(),
            &GenericShape::Not(ref n) => n.follows_rules(),
            &GenericShape::BoxCache(ref c) => c.follows_rules(),
            &GenericShape::Transformation(ref t) => t.follows_rules(),
            &GenericShape::Boxed(ref t) => t.follows_rules(),
            &GenericShape::Ref(ref t) => t.follows_rules(),
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
    fn follows_rules(&self) -> bool {
        self.target.follows_rules()
    }
}

impl <I: Implicit> Implicit for Not<I> {
    fn sample(&self, pos: Point) -> f32 {
        -self.target.sample(pos)
    }
    fn bounding_box(&self) -> Option<Rect> {
        None
    }
    fn follows_rules(&self) -> bool {
        self.target.follows_rules()
    }
}

impl Implicit for Polygon {
    fn sample(&self, pos: Point) -> f32 {
        const EPSILON: f32 = 0.0001;
        const ITERS: i32 = 3;

        fn is_inside(pos: Point, lines: &[Line]) -> bool {
            let ray = Ray(pos, Vector{x: rand::random(), y: rand::random()});
            // keep a list of these so we can tag duplicates
            let mut hit_points = vec![];
            for line in lines {
                if let Some(point) = ray.intersect_with_line(line) {
                    if !hit_points.iter().any(|p: &Point| p.close_to(&point, EPSILON)) {
                        hit_points.push(point);
                    }
                }
            }
            hit_points.len() % 2 == 0
        }

        let mut min = ::std::f32::INFINITY;
        for line in self.lines() {
            min = min.min(line.dist_to_point(&pos));
        }

        let mut inside_cnt = 0;
        for _ in 0 .. ITERS {
            if is_inside(pos, self.lines()) {
                inside_cnt += 1;
            }
        }

        let inside = inside_cnt > (ITERS / 2);

        if inside {
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
    fn follows_rules(&self) -> bool { true }
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
    fn follows_rules(&self) -> bool { true }
}

impl <A: Implicit, B: Implicit> Implicit for And<A, B> {
    fn sample(&self, pos: Point) -> f32 {
        self.left.sample(pos).max(self.right.sample(pos))
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

    fn follows_rules(&self) -> bool {
        self.left.follows_rules() &&
        self.right.follows_rules()
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

    fn follows_rules(&self) -> bool {
        self.left.follows_rules() &&
        self.right.follows_rules()
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

    fn follows_rules(&self) -> bool {
        self.left.follows_rules() &&
        self.right.follows_rules()
    }
}

impl <A: Implicit> Implicit for Boundary<A> {
    fn sample(&self, pos: Point) -> f32 {
        self.target.sample(pos) - self.move_by
    }

    fn bounding_box(&self) -> Option<Rect> {
        self.target.bounding_box().map(|r| r.expand(self.move_by, self.move_by, self.move_by, self.move_by))
    }

    fn follows_rules(&self) -> bool {
        self.target.follows_rules()
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

impl Implicit for Rectangle {
    fn sample(&self, pos: Point) -> f32 {
        self.poly.sample(pos)
    }

    fn bounding_box(&self) -> Option<Rect> {
        self.poly.bounding_box()
    }

    fn follows_rules(&self) -> bool { true }
}
