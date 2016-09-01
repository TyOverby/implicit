#![allow(dead_code)]
#![feature(pub_restricted, repr_simd)]

extern crate vecmath;
extern crate rand;
extern crate itertools;
extern crate flame;
extern crate fnv;
extern crate num_cpus;
extern crate simd;
extern crate rayon;
extern crate thread_id;

mod vectorize;
mod implementations;
pub mod formats;
mod scene;
pub(crate) mod util;

pub use vectorize::render::{render, RenderMode, OutputMode};
pub use formats::output_device::{OutputDevice, NullDevice};
pub use scene::*;
pub use vectorize::gather_lines;
pub use vectorize::line_gather;
pub use implementations::*;
pub mod geom {
    pub use ::util::geom::*;
}
pub mod quadtree {
    pub use ::util::quadtree::*;
}

use util::geom::{Point, Rect, Polygon};

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
        SyncBox::new(self)
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

    fn scale(self, factor: f32) -> Scale<Self> where Self: Sized {
        Scale {
            target: self,
            factor: factor,
        }
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

    fn fix_rules(self, recursion_depth: u32) -> PolyGroup where Self: Sized + Sync {
        let rendered = render(self, &RenderMode::Outline, recursion_depth, true);
        let lines = if let OutputMode::Outline(lines) = rendered {
            lines
        } else {
            panic!("somehow didn't get outline.");
        };

        PolyGroup {
            polys: lines.into_iter().map(|p| Polygon::new(p.into_iter())).collect()
        }
    }

    fn smooth(self, amount: f32, recursion_depth: u32) -> Boundary<PolyGroup> where Self: Sized + Sync {
        self.shrink(amount).fix_rules(recursion_depth).grow(amount)
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
