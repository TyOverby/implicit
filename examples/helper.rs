#![allow(dead_code)]

extern crate implicit;
extern crate lux;

use lux::prelude::*;
use lux::color;
use lux::interactive::Event;

use implicit::*;
use implicit::geom::*;

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

    fn render_lines(&self, shape: &SyncBox, frame: &mut Frame) {
        let paths = render(shape, &RenderMode::Outline, self.resolution, true);
        //          let colors = vec![lux::color::BLACK, lux::color::BLUE, lux::color::GREEN, lux::color::YELLOW, lux::color::BLACK, lux::color::RED];
        let colors = vec![lux::color::BLACK];
        let mut colors = colors.iter().cloned().cycle();

        let paths = match paths {
            OutputMode::Outline(p) => p,
            _ => panic!("not outline somehow!")
        };

        for path in paths {
            frame.color(colors.next().unwrap());
            let mut r = path;
            if let Some(last) = r.first().cloned() {
                r.push(last);
            }
            let pts = r.into_iter().map(|Point{x, y}| {
                let (dx, dy, _) = self.sample_to_draw((x, y));
                (dx, dy)
            });
            frame.draw_lines(pts, 1.0);
        }
    }

    fn draw_dots(&self, shape: &SyncBox, frame: &mut Frame) {
        for (sx, sy) in sampling_points(shape, self.resolution) {
            let (_, _, ds) = self.sample_to_draw((sx, sy));
            if ds > 5.0 {
                let (dpx, dpy, _) = self.sample_to_draw((sx, sy));
                let dot_size = ds / 5.0;
                let dot_offset = dot_size / 2.0;
                frame.square(dpx - dot_offset, dpy - dot_offset, dot_size)
                    .color(rgba(0.0, 0.0, 0.0, 0.5))
                    .fill();
            }
        }
    }

    fn render_pix(&self, shape: &SyncBox, frame: &mut Frame) {
        for (sx, sy) in sampling_points(shape, self.resolution) {
            let (dx, dy, ds) = self.sample_to_draw((sx, sy));
            let sample = shape.sample(Point { x: sx, y: sy } );

            let factor = 1.0;
            let color = if sample > 0.0 {
                rgba(sample / factor, 0.0, 0.0, 1.0)
            } else {
                rgba(0.0, -sample / factor, 0.0, 1.0)
            };
            frame.square(dx - 0.5 * ds, dy - 0.5 * ds, ds)
                .color(color)
                .fill();
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

    fn process_events(&mut self, window: &mut Window) -> bool {
        let mut dirty = false;
        for event in window.events() {
            match event {
                Event::WindowResized((x, y)) => {
                    let (x, y) = (x as f32, y as f32);
                    let m = x.min(y);
                    self.draw_scale = m / 100.0;
                    dirty = true;
                }
                Event::KeyReleased(_, Some('j'), _) => {
                    self.resolution *= 1.5f32;
                    dirty = true;
                }
                Event::KeyReleased(_, Some('k'), _) => {
                    self.resolution /= 1.5f32;
                    dirty = true;
                }
                Event::KeyReleased(_, Some('h'), _) => {
                    self.draw_scale *= 0.75;
                    dirty = true;
                }
                Event::KeyReleased(_, Some('l'), _) => {
                    self.draw_scale *= 1.5;
                    dirty = true;
                }
                _ => {}
            }
        }
        dirty

    }
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Display {
    Lines,
    Pixels,
    Dots,
}

pub fn display(resolution: f32, all: Vec<(SyncBox, Display)>) {
    let mut window = Window::new_with_defaults().unwrap();

    let mut canvas = ImplicitCanvas {
        draw_scale: 1.0,
        resolution: resolution,
    };

    let mut dirty = true;

    while window.is_open() {
        if dirty {
            let mut frame = window.cleared_frame(color::WHITE);

            for &(ref t, kind) in &all {
                let t: &SyncBox = t;
                if kind == Display::Lines {
                    canvas.render_lines(t, &mut frame);
                }
                if kind == Display::Pixels {
                    canvas.render_pix(t, &mut frame);
                }
                if kind == Display::Dots {
                    canvas.draw_dots(t, &mut frame);
                }
            }
        }

        dirty = canvas.process_events(&mut window);
    }
}

fn main() {
    panic!("not actually an example");
}
