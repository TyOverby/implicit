use ::Implicit;
use ::geom::{Rect, Point};

pub struct Bitmap {
    data: Vec<f32>,
    width: usize,
    height: usize,
    resolution: f32,
    offset: (f32, f32),
}

impl Bitmap {
    pub fn new<I: Implicit>(shape: &I, resolution: f32) -> Bitmap {
        let bb = shape.bounding_box().unwrap();
        let offset = (bb.top_left().x, bb.top_left().y);

        let min_x = bb.top_left().x.floor();
        let min_y = bb.top_left().y.floor();
        let max_x = bb.bottom_right().x.ceil();
        let max_y = bb.bottom_right().y.ceil();

        let rect = Rect::from_points(&Point{x: min_x, y: min_y}, &Point{x: max_x, y: max_y});

        let width = (rect.width() / resolution).ceil() as usize;
        let height = (rect.height() / resolution).ceil() as usize;

        let mut data = vec![0.0; width * height];
        for y in 0 .. height {
            for x in 0 .. width {
                data[ x + y * width] = shape.sample(Point {x: x as f32, y: y as f32});
            }
        }

        Bitmap {
            data: data,
            width: width,
            height: height,
            resolution: resolution,
            offset: offset,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.data[x + y * self.width]
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn sample<F>(&self, x: f32, y: f32, func: F) -> f32
    where F: FnOnce(f32, f32, f32, f32) -> f32 {
        let x_low = x.floor();
        // let x_low_err = x - x_low;
        let x_low = x_low as usize;

        let x_high = x.ceil();
        // let x_high_err = x_high - x;
        let x_high = x_high as usize;

        let y_low = y.floor();
        // let y_low_err = y - y_low;
        let y_low = y_low as usize;

        let y_high = y.ceil();
        // let y_high_err = y_high - y;
        let y_high = y_high as usize;

        let value_1 = self.get(x_low, y_low);
        let value_2 = self.get(x_low, y_high);
        let value_3 = self.get(x_high, y_low);
        let value_4 = self.get(x_high, y_high);
        func(value_1, value_2, value_3, value_4)
    }
}