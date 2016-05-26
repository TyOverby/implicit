use ::geom::Point;

pub trait OutputDevice {
    fn start_line(&mut self);
    fn add_point(&mut self, Point);
    fn end_line(&mut self);
    fn set_size(&mut self, _w: f32, _h: f32) { }
}

pub struct NullDevice;

impl OutputDevice for NullDevice {
    fn start_line(&mut self) {}
    fn add_point(&mut self, _: Point) {}
    fn end_line(&mut self) {}
    fn set_size(&mut self, _w: f32, _h: f32) { }
}
