extern crate implicit;
extern crate lux;

use std::vec;

use lux::prelude::*;
use lux::color;
use lux::interactive::Event;

use implicit::*;

struct ImplicitCanvas {
    draw_scale: f32,
    resolution: u32,
}

impl ImplicitCanvas {
    fn sampling_points(&self, shape: &Implicit) -> vec::IntoIter<(f32, f32)> {
        let bounding_box = shape.bounding_box().unwrap();
        let start = bounding_box.top_left;
        let end = bounding_box.bottom_right;
        let start_x = (start.x / self.resolution as f32) as u32 * self.resolution;
        let start_y = (start.y / self.resolution as f32) as u32 * self.resolution;
        let end_x = (end.x / self.resolution as f32) as u32 * self.resolution;
        let end_y = (end.y / self.resolution as f32) as u32 * self.resolution;


        let mut x = start_x;
        let mut y = start_y;
        let mut out = vec![];
        let res_scale = self.resolution as f32 * 0.5;

        while y < end_y {
            while x < end_x {
                out.push((x as f32 + res_scale, y as f32 + res_scale));
                x += self.resolution;
            }
            x = start_x;
            y += self.resolution;
        }
        out.into_iter()
    }

    fn sample_to_draw(&self, (x, y): (f32, f32)) -> (f32, f32, f32) {
        (x * self.draw_scale,
         y * self.draw_scale,
         self.resolution as f32 * self.draw_scale)
    }

    fn render_pix<S: Implicit>(&self, shape: &S, frame: &mut Frame) {
        for (sx, sy) in self.sampling_points(shape) {
            let (dx, dy, ds) = self.sample_to_draw((sx, sy));
            let sample = shape.sample(Point { x: sx, y: sy } );
            frame.square(dx - 0.5 * ds, dy - 0.5 * ds, ds)
                 .color(rgb(sample, sample, sample))
                 .fill();
            if ds > 5.0 {
                let (dpx, dpy, _) = self.sample_to_draw((sx, sy));
                let dot_size = ds / 5.0;
                let dot_offset = dot_size / 2.0;
                frame.square(dpx - dot_offset, dpy - dot_offset, dot_size)
                     .color(rgb(1.0, 0.0, 0.0))
                     .fill();
            }
        }

        self.draw_rect(&shape.bounding_box().unwrap(), frame);
    }

    fn draw_rect(&self, rect: &Rect, frame: &mut Frame) {
        frame.with_scale(self.draw_scale, self.draw_scale, |frame| {
            frame.rect(rect.top_left.x, rect.top_left.y, rect.width(), rect.height())
                 .border(1.0, (0.0, 0.0, 1.0))
                 .stroke();
        });
    }
}

fn main() {
    let mut window = Window::new_with_defaults().unwrap();

    let circle_1 = Circle {
        center: Point { x: 100.0, y: 100.0 },
        radius: 50.0
    };

    let circle_2 = Circle {
        center: Point { x:  150.0, y: 100.0 },
        radius: 50.0
    };

    let anded = Xor {
        left: circle_1,
        right: circle_2
    };

    let lone = Circle {
        center: Point { x: 125.0, y: 200.0 },
        radius: 50.0
    };

    struct Stripes;
    impl Implicit for Stripes {
        fn sample(&self, point: Point) -> f32 {
            point.x.sin() * point.y.sin()
        }

        fn bounding_box(&self) -> Option<Rect> { None }
    }

    let mut lone_and_stripes = And { left: Stripes, right: lone };

    let mut canvas = ImplicitCanvas {
        draw_scale: 1.0,
        resolution: 4,
    };

    while window.is_open() {
        let mut frame = window.cleared_frame(color::WHITE);
        canvas.render_pix(&anded, &mut frame);
        canvas.render_pix(&lone_and_stripes, &mut frame);

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

        lone_and_stripes.right.center.x += 1.0
    }

}
