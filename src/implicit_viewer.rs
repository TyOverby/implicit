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
    resolution: f32,
}

impl ImplicitCanvas {
    fn sample_to_draw(&self, (x, y): (f32, f32)) -> (f32, f32, f32) {
        (x * self.draw_scale,
         y * self.draw_scale,
         self.resolution as f32 * self.draw_scale)
    }

    fn render_lines<S: Implicit>(&self, shape: &S, frame: &mut Frame) {
        let mut scene = Scene::new(vec![GenericShape::Ref(shape)]);
        scene.resolution = self.resolution;
        let rendered_objects = scene.render(false);
        for RenderedObject(paths) in rendered_objects {
            println!("{} paths", paths.len());
            for path in paths {
                match path {
                    LineType::Joined(mut r) => {
                        if let Some(first) = r.first().cloned() {
                            r.push(first);
                        }
                        let pts = r.into_iter().map(|Point{x, y}| {
                            let (dx, dy, _) = self.sample_to_draw((x, y));
                            (dx, dy)
                        });
                        frame.draw_lines(pts, 1.0);
                    }
                    LineType::Unjoined(r) => {
                        let pts = r.into_iter().map(|Point{x, y}| {
                            let (dx, dy, _) = self.sample_to_draw((x, y));
                            (dx, dy)
                        });
                        frame.draw_lines(pts, 1.0);
                    }
                }
            }
        }
    }

    fn draw_dots<S: Implicit>(&self, shape: &S, frame: &mut Frame) {
        for (sx, sy) in sampling_points(shape.bounding_box().unwrap(), self.resolution) {
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
        for (sx, sy) in sampling_points(shape.bounding_box().unwrap(), self.resolution) {
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
        resolution: 4.0,
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
        canvas.draw_dots(&poly, &mut frame);

        for event in window.events() {
            match event {
                Event::WindowResized((x, y)) => {
                    let (x, y) = (x as f32, y as f32);
                    let m = x.min(y);
                    canvas.draw_scale = m / 100.0;
                }
                Event::KeyReleased(_, Some('j'), _) => {
                    canvas.resolution += 1f32;
                }
                Event::KeyReleased(_, Some('k'), _) => {
                    canvas.resolution -= 1f32;
                    if canvas.resolution <= 0.0 {
                        canvas.resolution = 1f32;
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
