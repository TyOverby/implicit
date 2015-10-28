extern crate implicit;
extern crate lux;

use lux::prelude::*;
use lux::color;
use lux::interactive::Event;

use implicit::*;

enum Mode {
    Draw,
    Query
}

fn draw_rectangle(frame: &mut Frame, rect: &Rect, color: (f32, f32, f32)) {
    let tl = rect.top_left;
    frame.rect(tl.x, tl.y, rect.width(), rect.height())
         .border(4.0, color)
         .stroke();
}

fn main() {
    let mut window = Window::new_with_defaults().unwrap();
    let mut last_down_position = None;
    let mut rects = vec![];
    let mut mode = Mode::Draw;
    let mut query = None;

    while window.is_open() {
        let cur_pos = window.mouse_pos();
        let mut this_rect = None;
        for event in window.events() {
            match event {
                Event::KeyReleased(_, Some('d'), _) => {
                    mode = Mode::Draw;
                }
                Event::KeyReleased(_, Some('q'), _) => {
                    mode = Mode::Query;
                }
                Event::MouseDown(_) => {
                    last_down_position = Some(window.mouse_pos());
                }
                Event::MouseUp(_) => {
                    if let Some(last_down) = last_down_position {
                        let rect = Rect::from_points(
                            &Point { x: last_down.0, y: last_down.1 },
                            &Point { x: cur_pos.0, y: cur_pos.1 }
                        );
                        this_rect = Some(rect);
                    }
                    last_down_position = None;
                }
                _ => {}
            }
        }

        match mode {
            Mode::Draw => {
                if let Some(r) = this_rect {
                    rects.push(r);
                }
            }
            Mode::Query => {
                if let Some(r) = this_rect {
                    query = Some(r);
                }
            }
        }

        let mut frame = window.cleared_frame(color::WHITE);
        for rect in &rects {
            if let Some(query) = query.as_ref() {
                if query.does_intersect(rect) {
                    draw_rectangle(&mut frame, rect, (0.0, 0.0, 0.5));
                } else {
                    draw_rectangle(&mut frame, rect, (0.0, 0.0, 0.0));
                }
            } else {
                draw_rectangle(&mut frame, rect, (0.0, 0.0, 0.0));
            }
        }
        if let Some(query) = query.as_ref() {
            draw_rectangle(&mut frame, query, (0.0, 0.5, 0.0));
        }
    }

}
