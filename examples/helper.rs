#![allow(dead_code)]

extern crate implicit;
extern crate lux;

use lux::prelude::*;
use lux::color;
use lux::interactive::Event;

use implicit::*;
use implicit::geom::*;

struct ImplicitCanvas {
    recurse_limit: u32,
    draw_scale: f32,
    debug: bool,
    draw_offset: (f32, f32),

    drag_start: Option<(f32, f32)>,
}

impl ImplicitCanvas {
    fn process_events(&mut self, window: &mut Window) -> bool {
        let mut dirty = false;
        for event in window.events() {
            match event {
                Event::MouseDown(_) => {
                    self.drag_start = Some(window.mouse_pos())
                }
                Event::MouseUp(_) => {
                    self.drag_start = None;
                }
                Event::MouseMoved((x, y)) => {
                    let (x, y) = (x as f32, y as f32);
                    if let Some((xs, ys)) = self.drag_start {
                        let dx = (x - xs) / self.draw_scale;
                        let dy = (y - ys) / self.draw_scale;

                        self.draw_offset.0 += dx;
                        self.draw_offset.1 += dy;
                        self.drag_start = Some((x, y));
                        dirty = true;
                    }
                }
                Event::WindowResized((x, y)) => {
                    let (x, y) = (x as f32, y as f32);
                    let m = x.min(y);
                    self.draw_scale = m / 100.0;
                    dirty = true;
                }
                Event::KeyReleased(_, Some('j'), _) => {
                    self.recurse_limit += 1;
                    dirty = true;
                }
                Event::KeyReleased(_, Some('k'), _) => {
                    if self.recurse_limit != 1 {
                        self.recurse_limit -= 1;
                        dirty = true;
                    }
                }
                Event::KeyReleased(_, Some('h'), _) => {
                    self.draw_scale *= 0.75;
                    dirty = true;
                }
                Event::KeyReleased(_, Some('l'), _) => {
                    self.draw_scale *= 1.5;
                    dirty = true;
                }
                Event::KeyReleased(_, Some('d'), _) => {
                    self.debug = !self.debug;
                    dirty = true;
                }
                _ => {}
            }
        }
        dirty
    }
}

fn fill(canvas: &mut Frame, rect: Rect, color: [f32; 3]) {
    let Point{x, y} = rect.top_left();
    let (w, h) = (rect.width(), rect.height());
    canvas.rect(x, y, w, h).color(color).fill();
}

fn fill_partial(canvas: &mut Frame, rect: Rect, fill_value: f32) {
    let furthest = {
        let r = rect.width() / 2.0;
        let _2r2 = 2.0 * (r * r);
        _2r2.sqrt()
    };
    let c = (fill_value + furthest) / (2.0 * furthest);
    let color = [c, c, c];

    fill(canvas, rect, color);
}

pub fn display(scene: &mut Scene) {
    let mut window = Window::new_with_defaults().unwrap();

    let mut canvas = ImplicitCanvas {
        draw_scale: 1.0,
        recurse_limit: 5,
        debug: false,
        draw_offset: (0.0, 0.0),
        drag_start: None,
    };

    let mut dirty = true;

    while window.is_open() {
        if dirty {
            let mut frame = window.cleared_frame(color::WHITE);
            frame.scale(canvas.draw_scale, canvas.draw_scale);
            frame.translate(canvas.draw_offset.0, canvas.draw_offset.1);
            scene.recursion_depth = canvas.recurse_limit;
            scene.sample_all(|rect, sample_value| {
                match sample_value {
                    SampleValue::Filled => {
                        fill(&mut frame, rect, [0.0, 0.0, 0.0]);
                    }
                    SampleValue::Empty => {},
                    SampleValue::Partial(fill) => {
                        fill_partial(&mut frame, rect, fill);
                    }
                }
            });
        }

        dirty = canvas.process_events(&mut window);
    }
}

fn main() {
    panic!("not actually an example");
}
