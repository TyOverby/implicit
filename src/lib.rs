mod geom;
pub use geom::*;

pub trait Implicit {
    fn sample(&self, pos: Point) -> Scalar;
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
    pub radius: Scalar
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
    fn sample(&self, pos: Point) -> Scalar {
        (**self).sample(pos)
    }

    fn bounding_box(&self) -> Rect {
        (**self).bounding_box()
    }
}

impl Implicit for GenericShape {
    fn sample(&self, pos: Point) -> Scalar {
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
    fn sample(&self, pos: Point) -> Scalar {
        let Point(a, b) = pos;
        let Point(c, d) = self.center;
        let dx = a - c;
        let dy = b - d;
        let dist = (dx * dx + dy * dy).sqrt();
        Scalar(dist - self.radius.0)
    }

    fn bounding_box(&self) -> Rect {
        let Point(cx, cy) = self.center;
        let Scalar(r) = self.radius;
        Rect(
            Point(cx - r, cy - r),
            Point(cx + r, cy + r))
    }
}

impl <A: Implicit, B: Implicit> Implicit for And<A, B> {
    fn sample(&self, pos: Point) -> Scalar {
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
    fn sample(&self, pos: Point) -> Scalar {
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
    fn sample(&self, pos: Point) -> Scalar {
        let left_sample = self.left.sample(pos);
        let right_sample = self.right.sample(pos);

        if left_sample.0 < 0.0 && right_sample.0 < 0.0 {
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
