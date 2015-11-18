#![allow(dead_code)]

extern crate implicit;
extern crate lux;

mod examples;

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
        let start_x = (start.x / self.resolution as f32) as u32 * self.resolution - 1;
        let start_y = (start.y / self.resolution as f32) as u32 * self.resolution - 1;
        let end_x   = (end.x / self.resolution as f32) as u32 * self.resolution + 1;
        let end_y   = (end.y / self.resolution as f32) as u32 * self.resolution + 1;


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

    fn render_lines<S: Implicit>(&self, shape: &S, frame: &mut Frame) {
        let mut lines = vec![];
        for (sx, sy) in self.sampling_points(shape) {
            match march(shape, Point {x: sx, y: sy}, self.resolution as f32) {
                MarchResult::None => {},
                MarchResult::OneDebug(line) => {
                    let (dx, dy, ds) = self.sample_to_draw((sx, sy));
                    frame.square(dx - ds / 2.0, dy - ds / 2.0, ds)
                         .color(rgb(0.0, 1.0, 0.0))
                         .fill();
                    lines.push(line)
                }
                MarchResult::One(line) => lines.push(line),
                MarchResult::Two(line1, line2) => {
                    lines.push(line1);
                    lines.push(line2);
                }
                MarchResult::Debug => {
                    let (dx, dy, ds) = self.sample_to_draw((sx, sy));
                    frame.square(dx - ds / 2.0, dy - ds / 2.0, ds)
                         .color(rgb(0.0, 1.0, 0.0))
                         .fill();
                }
            }
        }

        /*
        for line in &lines {
            let (ax, ay, _) = self.sample_to_draw((line.0.x, line.0.y));
            let (bx, by, _) = self.sample_to_draw((line.1.x, line.1.y));
            frame.draw_line(ax, ay, bx, by, 1.0);
        }*/

        let res = self.resolution as f32 / 2.0;
        let (simplified, tree) = connect_lines(lines, res);

        tree.inspect(|rect, _, _| {
            self.draw_rect(rect, frame);
        });

        for line_type in simplified.into_iter() {
            let points = match line_type {
                LineType::Unjoined(points) => points,
                LineType::Joined(mut points) => {
                    if let Some(first) = points.first().cloned() {
                        points.push(first);
                    }
                    println!("points in shape: {}", points.len());
                    points
                },
            };

            let screen_points = points.into_iter().map(|point| {
                let (a, b, _) = self.sample_to_draw((point.x, point.y));
                (a, b)
            });

            frame.draw_lines(screen_points, 1.0);
        }
    }

    fn draw_dots<S: Implicit>(&self, shape: &S, frame: &mut Frame) {
        for (sx, sy) in self.sampling_points(shape) {
            let (_, _, ds) = self.sample_to_draw((sx, sy));
            if ds > 5.0 {
                let (dpx, dpy, _) = self.sample_to_draw((sx, sy));
                let dot_size = ds / 5.0;
                let dot_offset = dot_size / 2.0;
                frame.square(dpx - dot_offset, dpy - dot_offset, dot_size)
                     .color(rgb(1.0, 0.0, 0.0))
                     .fill();
            }
        }
    }

    fn render_pix<S: Implicit>(&self, shape: &S, frame: &mut Frame) {
        for (sx, sy) in self.sampling_points(shape) {
            let (dx, dy, ds) = self.sample_to_draw((sx, sy));
            let sample = shape.sample(Point { x: sx, y: sy } );

            let factor = 10.0;
            let color = if sample > 0.0 {
                rgba(sample / factor, 0.0, 0.0, 0.5)
            } else {
                rgba(0.0, -sample / factor, 0.0, 0.5)
            };
            //let color = rgba(sample, sample, sample, -sample);
            frame.square(dx - 0.5 * ds, dy - 0.5 * ds, ds)
                 .color(color)
                 .fill();
        }
        //self.draw_rect(&shape.bounding_box().unwrap(), frame);
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

    let xored = examples::xored_circles();
    let mut stripes = examples::stripes();
    let poly = examples::poly();
//    let poly = examples::rect();

    let modified = Boundary {
        target: poly.clone(),
        move_by: -30.0
    };

    let mut canvas = ImplicitCanvas {
        draw_scale: 1.0,
        resolution: 4,
    };

    while window.is_open() {
        let mut frame = window.cleared_frame(color::WHITE);
//        canvas.render_pix(&modified, &mut frame);
//        canvas.render_pix(&stripes, &mut frame);
//        canvas.render_pix(&poly, &mut frame);

//        canvas.render_lines(&modified, &mut frame);
//        canvas.render_lines(&stripes, &mut frame);
        canvas.render_lines(&poly, &mut frame);

//        canvas.draw_dots(&modified, &mut frame);
//        canvas.draw_dots(&poly, &mut frame);

        for event in window.events() {
            match event {
                Event::WindowResized((x, y)) => {
                    let (x, y) = (x as f32, y as f32);
                    let m = x.min(y);
                    canvas.draw_scale = m / 100.0;
                }
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
                    canvas.draw_scale *= 0.75;
                }
                Event::KeyReleased(_, Some('l'), _) => {
                    canvas.draw_scale *= 1.5;
                }
                _ => {}
            }
        }

        stripes.right.center.x += 1.0
    }

}
