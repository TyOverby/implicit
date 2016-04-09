#![allow(dead_code)]

extern crate vecmath;
extern crate rand;
extern crate itertools;
extern crate crossbeam;
extern crate flame;
extern crate fnv;
extern crate num_cpus;
extern crate simd;

mod private;
mod scene;

pub use private::render::{render, RenderMode, OutputMode};
pub use private::sample_points::sampling_points;
pub use private::geom;
pub use private::quadtree::{Spatial, QuadTree};
pub use private::output_device::OutputDevice;
pub use scene::Scene;
pub use private::svg::SvgWriter;
pub use private::pdf::PdfWriter;


use private::{Rect, Point, Polygon, Matrix, Vector, Line, Ray};
use std::rc::Rc;

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

    /// True if the shape follows all the rules about implicit shapes.
    fn follows_rules(&self) -> bool;

    fn boxed(self) -> SyncBox where Self: Sized + 'static + Sync {
        SyncBox { inner: Rc::new(self) }
    }

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
        r.matrix = r.matrix.scale(sx, sy);
        r
    }

    fn translate(self, x: f32, y: f32) -> Transformation<Self> where Self: Sized {
        let mut r = Transformation::new(self);
        r.matrix = r.matrix.translate(x, y);
        r
    }

    fn rotate(self, rads: f32) -> Transformation<Self> where Self: Sized {
        let mut r = Transformation::new(self);
        r.matrix = r.matrix.rotate(rads);
        r
    }

    fn not(self) -> Not<Self> where Self: Sized {
        Not { target: self}
    }

    fn outline_inner(self, distance: f32) -> And<Self, Not<Boundary<Self>>> where Self: Sized + Clone {
        self.clone().and(self.shrink(distance).not())
    }

    fn outline_outer(self, distance: f32) -> And<Boundary<Self>, Not<Self>> where Self: Sized + Clone {
        self.clone().grow(distance).and(self.not())
    }

    fn borrow<'a>(&'a self) -> &'a Implicit where Self: Sized {
        self
    }

    fn fix_rules(self, resolution: f32) -> PolyGroup where Self: Sized + Sync {
        let rendered = render(self, &RenderMode::Outline, resolution, true);
        let lines = if let OutputMode::Outline(lines) = rendered {
            lines
        } else {
            panic!("somehow didn't get outline.");
        };

        PolyGroup {
            polys: lines.into_iter().map(|p| Polygon::new(p.into_iter())).collect()
        }
    }

    fn smooth(self, amount: f32, resolution: f32) -> Boundary<PolyGroup> where Self: Sized + Sync {
        self.shrink(amount).fix_rules(resolution).grow(amount)
    }

    fn center(&self) -> Option<Point> {
        self.bounding_box().map(|a|Rect::midpoint(&a))
    }

    fn center_at(self, point: &Point) -> Transformation<Self> where Self: Sized {
        let my_center = self.center().unwrap();
        let delta = *point - my_center;
        self.translate(delta.x, delta.y)
    }
}

pub trait SyncImplicit: Sync + Implicit { }

impl <A> SyncImplicit for A where A: Implicit + Sync {}

#[derive(Clone, Debug)]
pub struct PolyGroup {
    pub polys: Vec<Polygon>
}

#[derive(Copy, Clone, Debug)]
pub struct Circle {
    pub center: Point,
    pub radius: f32
}

#[derive(Clone, Debug)]
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

impl <'a, A> Implicit for &'a A where A: Implicit + Sized {
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

#[derive(Clone)]
pub struct SyncBox {
    inner: Rc<Implicit + Sync>
}

impl Implicit for SyncBox {
    fn sample(&self, pos: Point) -> f32 {
        self.inner.sample(pos)
    }

    fn bounding_box(&self) -> Option<Rect> {
        self.inner.bounding_box()
    }

    fn follows_rules(&self) -> bool {
        self.inner.follows_rules()
    }
}

unsafe impl Sync for SyncBox { }

/*
impl <A: Implicit + Sync + Send> Implicit for Arc<A> {
    fn sample(&self, pos: Point) -> f32 {
        (**self).sample(pos)
    }

    fn bounding_box(&self) -> Option<Rect> {
        (**self).bounding_box()
    }

    fn follows_rules(&self) -> bool {
        (**self).follows_rules()
    }
}*/

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
        use simd::*;
        use std::cmp::min;

        // Minimum distance.
        let (min, inside) = {
            let mut left_xs = self.left_xs();
            let mut left_ys = self.left_ys();
            let mut right_xs = self.right_xs();
            let mut right_ys = self.right_ys();
            // Even though this is a noop, we do this to trick LLVM into optimizing away bounds checks.
            let min_len = min(min(left_xs.len(), left_ys.len()),min(right_xs.len(), right_ys.len()));
            left_xs = &left_xs[.. min_len];
            left_ys = &left_ys[.. min_len];
            right_xs = &right_xs[.. min_len];
            right_ys = &right_ys[.. min_len];

            let pos_x = f32x4::splat(pos.x);
            let pos_y = f32x4::splat(pos.y);
            let ray_1 = Ray(pos, Vector{x: 1.0, y: 0.0});
            let ray_2 = Ray(pos, Vector{x: 1.0, y: 1.0});
            let ray_3 = Ray(pos, Vector{x: 0.0, y: 1.0});
            let ray_1x = f32x4::splat(ray_1.1.x);
            let ray_1y = f32x4::splat(ray_1.1.y);
            let ray_2x = f32x4::splat(ray_2.1.x);
            let ray_2y = f32x4::splat(ray_2.1.y);


            let mut min_dist = f32x4::splat(::std::f32::INFINITY);
            let mut ray_intersect_1 = i32x4::splat(0);
            let mut ray_intersect_2 = i32x4::splat(0);

            let mut simd_used = 0;
            while left_xs.len() >= 4 && left_ys.len() >= 4 && right_xs.len() >= 4 && right_ys.len() >= 4 {
                let wx = f32x4::load(left_xs, 0);
                let wy = f32x4::load(left_ys, 0);

                let vx = f32x4::load(right_xs, 0);
                let vy = f32x4::load(right_ys, 0);

                min_dist = min_dist.min(::geom::simd::line_to_point_simd(pos_x, pos_y, vx, vy, wx, wy));
                let (r1, r2) = ::geom::simd::lines_touching_rays(pos_x, pos_y, ray_1x, ray_1y, ray_2x, ray_2y, vx, vy, wx, wy);
                ray_intersect_1 = ray_intersect_1 + r1;
                ray_intersect_2 = ray_intersect_2 + r2;

                left_xs = &left_xs[4 ..];
                left_ys = &left_ys[4 ..];
                right_xs = &right_xs[4 ..];
                right_ys = &right_ys[4 ..];
                simd_used += 4;
            }

            // Min dist
            let mut out_dist = [0.0, 0.0, 0.0, 0.0];
            min_dist.store(&mut out_dist, 0);
            let mut min = out_dist[0].min(out_dist[1]).min(out_dist[2]).min(out_dist[3]);
            for line in &self.lines()[simd_used..] {
                min = min.min(line.dist_to_point_2(pos));
            }

            // Intersect count
            let mut out_intersect_1 = [0, 0, 0, 0];
            let mut out_intersect_2 = [0, 0, 0, 0];
            ray_intersect_1.store(&mut out_intersect_1, 0);
            ray_intersect_2.store(&mut out_intersect_2, 0);
            let mut out_intersect_1 = out_intersect_1[0] + out_intersect_1[1] + out_intersect_1[2] + out_intersect_1[3];
            let mut out_intersect_2 = out_intersect_2[0] + out_intersect_2[1] + out_intersect_2[2] + out_intersect_2[3];

            fn intersection_count_dummy(ray: Ray, lines: &[Line]) -> i32 {
                let mut hit_count = 0;
                for line in lines {
                    if ray.does_intersect_with_line(line) {
                        hit_count += 1;
                    }
                }
                hit_count 
            }

            out_intersect_1 += intersection_count_dummy(ray_1, &self.lines()[simd_used..]);
            out_intersect_2 += intersection_count_dummy(ray_2, &self.lines()[simd_used..]);

            //debug_assert_eq!(out_intersect_1, intersection_count_dummy(ray_1, self.lines()));
            //debug_assert_eq!(out_intersect_2, intersection_count_dummy(ray_2, self.lines()));

            let inside = if out_intersect_1 % 2  == out_intersect_2 % 2 {
                out_intersect_1 % 2 == 0
            } else {
                intersection_count_dummy(ray_3, self.lines()) % 2 == 0
            };

            (min, inside)
        };

        if inside {
            min.sqrt()
        } else {
            -min.sqrt()
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
