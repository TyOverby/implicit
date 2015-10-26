extern crate implicit;
extern crate lux;

use lux::prelude::*;
use lux::color;

use implicit::*;

struct ImplicitCanvas {
    size: (u32, u32),
    pix_width: u32,
}

impl ImplicitCanvas {
    fn render<S: Implicit>(&self, shape: &S, frame: &mut Frame) {
        let mut sx = 0.5f32;
        let mut sy = 0.5f32;
        let dist = self.pix_width as f32;
        let ex = dist * self.size.0 as f32;
        let ey = dist * self.size.1 as f32;

        while sy < ey {
            while sx < ex {
                let sample = shape.sample(Point(sx / dist, sy / dist));
                frame.square(sx, sy, dist)
                     .color(rgb(sample.0, sample.0, sample.0))
                     .fill();
                sx += dist;
            }
            sx = 0.5;
            sy += dist;
        }
    }
}

fn main() {
    let mut window = Window::new_with_defaults().unwrap();

    let circle_1 = Circle {
        center: Point(100.0, 100.0),
        radius: Scalar(50.0)
    };

    let circle_2 = Circle {
        center: Point(150.0, 100.0),
        radius: Scalar(50.0)
    };

    let anded = Xor {
        left: circle_1,
        right: circle_2
    };

    let canvas = ImplicitCanvas {
        size: (200, 200),
        pix_width: 3
    };

    while window.is_open() {
        let mut frame = window.cleared_frame(color::WHITE);
        println!("hi");
        canvas.render(&anded, &mut frame);
        println!("bye");
    }
}
