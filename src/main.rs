extern crate implicit;
extern crate lux;

use lux::prelude::*;
use lux::graphics::ColorVertex;
use lux::color;

use implicit::*;

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

    let anded = And {
        left: circle_1,
        right: circle_2
    };

    // Set up the point buffer
    let mut points = Vec::with_capacity(255 * 255);
    for x in 0 .. 255 {
        for y in 0 .. 255 {
            points.push(ColorVertex {
                pos: [x as f32, y as f32],
                color: rgb(1.0, 1.0, 1.0)
            });
        }
    }

    while window.is_open() {
        let mut frame = window.cleared_frame(color::WHITE);
        // Update the point buffer with a new noise pattern
        for pt in &mut points {
            let x = pt.pos[0];
            let y = pt.pos[1];
            if anded.sample(Point(x, y)).0 < 0f32 {
                pt.color = rgb(0, 0, 0);
            } else {
                pt.color = rgb(255, 0, 0);
            }
        }

        frame.draw_points(&points);
    }
}
