extern crate implicit;
extern crate lux;

use std::vec;

use lux::prelude::*;
use lux::color;
use lux::interactive::Event;

use implicit::*;

struct ImplicitCanvas {
    draw_scale: f32,
    size: (u32, u32),
    resolution: u32,
}

impl ImplicitCanvas {
    fn sampling_points(&self) -> vec::IntoIter<(f32, f32)> {
        let mut x = 0;
        let mut y = 0;
        let mut out = vec![];

        while y < self.size.1 {
            while x < self.size.0 {
                out.push((x as f32 + 0.5, y as f32 + 0.5));
                x += self.resolution;
            }
            x = 0;
            y += self.resolution;
        }
        out.into_iter()
    }

    fn sample_to_draw(&self, (x, y): (f32, f32)) -> (f32, f32, f32) {
        ((x - 0.5) * self.draw_scale,
         (y - 0.5) * self.draw_scale,
         self.resolution as f32 * self.draw_scale)
    }

    fn render_pix<S: Implicit>(&self, shape: &S, frame: &mut Frame) {
        let dist = self.resolution as f32;
        for (sx, sy) in self.sampling_points() {
            let (dx, dy, ds) = self.sample_to_draw((sx, sy));
            let sample = shape.sample(Point(sx, sy));
            frame.square(dx, dy, ds)
                 .color(rgb(sample.0, sample.0, sample.0))
                 .fill();
            if ds > 5.0 {
                let (dpx, dpy, _) = self.sample_to_draw((sx, sy));
                let (dpx, dpy) = (dpx + 0.5 * self.draw_scale * self.resolution as f32,
                                  dpy + 0.5 * self.draw_scale * self.resolution as f32);
                frame.square(dpx, dpy, 1.0)
                     .color(rgb(1.0, 0.0, 0.0))
                     .fill();
            }
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

    let mut canvas = ImplicitCanvas {
        draw_scale: 1.0,
        size: (200, 200),
        resolution: 4
    };

    while window.is_open() {
        let mut frame = window.cleared_frame(color::WHITE);
        canvas.render_pix(&anded, &mut frame);

        for event in window.events() {
            match event {
                Event::KeyReleased(_, Some('j'), _) => {
                    canvas.resolution += 1;
                }
                Event::KeyReleased(_, Some('k'), _) => {
                    canvas.resolution -= 1;
                    if canvas.resolution == 0 {
                        canvas.resolution = 1;
                    }
                }
                Event::KeyReleased(_, Some('h'), _) => {
                    canvas.draw_scale *= 0.5;
                }
                Event::KeyReleased(_, Some('l'), _) => {
                    canvas.draw_scale *= 2.0;
                }
                _ => {}
            }
        }
    }
}
