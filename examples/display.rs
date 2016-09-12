#![allow(dead_code)]

extern crate implicit;
extern crate lux;

use lux::prelude::*;
use lux::interactive::Event;

use implicit::*;
use implicit::geom::Point;

struct DisplayCanvas<'a> {
    zoom: f32,
    scene: Scene<'a>,
}

fn handle_events<I>(canvas: &mut DisplayCanvas, events: I, dirty: &mut bool)
where I: Iterator<Item=Event> {
    for event in events {
        match event {
            Event::WindowResized((x, y)) => {
                let (x, y) = (x as f32, y as f32);
                let m = x.min(y);
                canvas.zoom = m / 100.0;
                *dirty = true;
            }
            Event::KeyReleased(_, Some('j'), _) => {
                canvas.scene.recursion_depth += 1;
                *dirty = true;
            }
            Event::KeyReleased(_, Some('k'), _) => {
                if canvas.scene.recursion_depth != 0 {
                    canvas.scene.recursion_depth -= 1;
                }
                *dirty = true;
            }
            Event::KeyReleased(_, Some('h'), _) => {
                canvas.zoom *= 0.75;
                *dirty = true;
            }
            Event::KeyReleased(_, Some('l'), _) => {
                canvas.zoom *= 1.5;
                *dirty = true;
            }
            _ => {}
        }
    }
}

pub struct DrawStateHolder<'a>(&'a mut DrawState, &'a mut Frame, f32);

#[derive(Copy, Clone)]
pub enum DrawState {
    Start,
    Middle(f32, f32),
}

impl <'a> OutputDevice for DrawStateHolder<'a> {
    fn start_line(&mut self) {
        *self.0 = DrawState::Start;
    }

    fn add_point(&mut self, Point{x, y}: Point) {
        let s = self.2;
        if let DrawState::Middle(px, py) = *self.0 {
            self.1.draw_line(px * s, py * s, x * s, y * s, 1.0);
        }
        *self.0 = DrawState::Middle(x, y);
    }

    fn end_line(&mut self) {
        *self.0 = DrawState::Start;
    }
}

pub fn display<'a>(scene: Scene<'a>) {
    let mut canvas = DisplayCanvas {
        zoom: 1.0,
        scene: scene,
    };
    let mut window = Window::new_with_defaults().unwrap();
    let mut dirty = true;

    loop {
        handle_events(&mut canvas, window.events(), &mut dirty);
        if dirty {
            dirty = false;
            let mut frame = window.cleared_frame(rgb(1.0, 1.0, 1.0));
            canvas.scene.render_all(&mut DrawStateHolder(&mut DrawState::Start, &mut frame, canvas.zoom));
        }
    }
}

fn main() {
    panic!("not actually an example");
}
