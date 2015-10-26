mod geom;
pub use geom::*;

pub trait Implicit {
    fn sample(&self, pos: Point) -> f32;
    fn bounding_box(&self) -> Rect;
}

pub enum GenericShape {
    Circle(Circle),
    And(And<Box<GenericShape>, Box<GenericShape>>),
    Or(Or<Box<GenericShape>, Box<GenericShape>>),
    Xor(Xor<Box<GenericShape>, Box<GenericShape>>),
}

pub struct Circle {
    pub center: Point,
    pub radius: f32
}

pub struct And<A: Implicit, B: Implicit> {
    pub left: A,
    pub right: B,
}

pub struct Or<A: Implicit, B: Implicit> {
    pub left: A,
    pub right: B,
}

pub struct Xor<A: Implicit, B: Implicit> {
    pub left: A,
    pub right: B,
}

impl <A: Implicit> Implicit for Box<A> {
    fn sample(&self, pos: Point) -> f32 {
        (**self).sample(pos)
    }

    fn bounding_box(&self) -> Rect {
        (**self).bounding_box()
    }
}

impl Implicit for GenericShape {
    fn sample(&self, pos: Point) -> f32 {
        match self {
            &GenericShape::Circle(ref circle) => circle.sample(pos),
            &GenericShape::And(ref and) => and.sample(pos),
            &GenericShape::Or(ref or) => or.sample(pos),
            &GenericShape::Xor(ref xor) => xor.sample(pos),
        }
    }

    fn bounding_box(&self) -> Rect {
        match self {
            &GenericShape::Circle(ref circle) => circle.bounding_box(),
            &GenericShape::And(ref and) => and.bounding_box(),
            &GenericShape::Or(ref or) => or.bounding_box(),
            &GenericShape::Xor(ref xor) => xor.bounding_box(),
        }
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

    fn bounding_box(&self) -> Rect {
        let Point{x: cx, y: cy} = self.center;
        let r = self.radius;
        Rect {
            top_left: Point {
                x: cx - r,
                y: cy - r
            },
            bottom_right: Point{
                x: cx + r,
                y: cy + r
            }
        }
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

    fn bounding_box(&self) -> Rect {
        // TODO: actually and the bounding boxes
        self.left.bounding_box()
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

    fn bounding_box(&self) -> Rect {
        // TODO: actually and the bounding boxes
        self.left.bounding_box()
    }
}

impl <A: Implicit, B: Implicit> Implicit for Xor<A, B> {
    fn sample(&self, pos: Point) -> f32 {
        let left_sample = self.left.sample(pos);
        let right_sample = self.right.sample(pos);

        if left_sample < 0.0 && right_sample < 0.0 {
            if -left_sample < -right_sample {
                -left_sample
            } else {
                -right_sample
            }
        } else {
            if left_sample < right_sample {
                left_sample
            } else {
                right_sample
            }
        }
    }

    fn bounding_box(&self) -> Rect {
        // TODO: actually and the bounding boxes
        self.left.bounding_box()
    }
}
