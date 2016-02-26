pub trait OutputDevice {
    fn start_line(&mut self);
    fn add_point(&mut self, x: f32, y: f32);
    fn end_line(&mut self);
    fn set_size(&mut self, w: f32, h: f32) { }
}
