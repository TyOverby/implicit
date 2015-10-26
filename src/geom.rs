#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub struct Scalar(pub f32);

#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub struct Point(pub f32, pub f32);

#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub struct Line(pub Point, pub Point);

#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub struct Rect(pub Point, pub Point);
