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

struct FrameWrapper<'a>(&'a mut Frame, f32, bool);

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

impl <'a> FrameWrapper<'a> {
    fn fill(&mut self, rect: Rect, color: [f32; 3]) {
        let Point{x, y} = rect.top_left();
        let (w, h) = (rect.width(), rect.height());
        self.0.rect(x, y, w, h).color(color).fill();
    }
    fn fill_partial(&mut self, rect: Rect, fill: f32) {
        let furthest = {
            let r = rect.width() / 2.0;
            let _2r2 = 2.0 * (r * r);
            _2r2.sqrt()
        };
        let c = (fill + furthest) / (2.0 * furthest);
        let color = [c, c, c];

        self.fill(rect, color);
    }
}

impl <'a> line_gather::QuadTreeProducer for FrameWrapper<'a> {
    type Tree = ();

    fn make_leaf_full(&mut self, rect: Rect) -> Self::Tree {
        let color = if self.2 { [ 1.0, 0.0, 0.0 ] } else { [0.0, 0.0, 0.0] };
        self.fill(rect, color);
    }
    fn make_leaf_empty(&mut self, _rect: Rect) -> Self::Tree {
        /*
        let color = if self.2 { [ 0.0, 1.0, 0.0 ] } else { [1.0, 1.0, 1.0] };
        self.fill(rect, color)
        */
    }
    fn make_leaf_line(&mut self, rect: Rect, fill: f32, Line(Point{x: p1x, y: p1y}, Point{x: p2x, y: p2y}): Line) -> Self::Tree {
        if self.2 {
            let color = [0.0, 0.0, 0.0];
            self.fill(rect, color);
            let res = self.1;
            self.0.with_color([1.0, 1.0, 1.0], |frame| {
                frame.draw_line(p1x, p1y, p2x, p2y, 1.0 / res);
            });

        } else {
            self.fill_partial(rect, fill)
        }

    }
    fn make_leaf_double_line(&mut self, rect: Rect, fill: f32, _l1: Line, _l2: Line) -> Self::Tree {
        if self.2 {
            self.fill(rect, [0.0, 1.0, 1.0]);
        } else {
            self.fill_partial(rect, fill)
        }
    }
    fn make_branch(&mut self, _rect: Rect, _a: Self::Tree, _b: Self::Tree, _c: Self::Tree, _d: Self::Tree) -> Self::Tree {

    }
    fn make_empty(&mut self, rect: Rect, fill: f32) -> Self::Tree {
        if self.2 {
            self.fill(rect, [0.0, 0.0, 1.0]);
        } else {
            self.fill_partial(rect, fill)
        }
    }
}

pub fn display<I: Implicit + Sync + ?Sized>(shapes: &[&I]) {
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
            for &shape in shapes {
                line_gather::gather_lines(&mut FrameWrapper(&mut frame, canvas.draw_scale, canvas.debug), shape, canvas.recurse_limit);
            }
        }

        dirty = canvas.process_events(&mut window);
    }
}

fn main() {
    panic!("not actually an example");
}
