extern crate implicit;
extern crate lux;

use lux::prelude::*;
use lux::color;
use lux::interactive::Event;

use implicit::*;

#[derive(Eq, PartialEq, Clone, Copy)]
enum AddMode {
    Draw,
    Query
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum QueryMode {
    List,
    Tree
}

fn draw_rectangle(frame: &mut Frame, rect: &Rect, color: (f32, f32, f32)) {
    let tl = rect.top_left;
    frame.rect(tl.x, tl.y, rect.width(), rect.height())
         .border(4.0, color)
         .stroke();
}

fn draw_help_text(frame: &mut Frame, add_mode: AddMode, query_mode: QueryMode) {
    frame.text("[d]raw add_mode", 20.0, 20.0)
         .color(if add_mode == AddMode::Draw { (0.6, 0.0, 0.6) } else { (0.0, 0.0, 0.0) })
         .draw().unwrap();
    frame.text("[q]uery add_mode", 20.0, 40.0)
         .color(if add_mode == AddMode::Query { (0.6, 0.0, 0.6) } else { (0.0, 0.0, 0.0) })
         .draw().unwrap();
    frame.text("[l]ist query", 20.0, 80.0)
         .color(if query_mode == QueryMode::List { (0.6, 0.0, 0.6) } else { (0.0, 0.0, 0.0) })
         .draw().unwrap();
    frame.text("[t]ree query", 20.0, 100.0)
         .color(if query_mode == QueryMode::Tree { (0.6, 0.0, 0.6) } else { (0.0, 0.0, 0.0) })
         .draw().unwrap();
}

fn main() {
    let mut window = Window::new_with_defaults().unwrap();
    let mut last_down_position = None;
    let mut rects = vec![];
    let mut add_mode = AddMode::Draw;
    let mut query_mode = QueryMode::List;
    let mut query = None;

    while window.is_open() {
        let cur_pos = window.mouse_pos();
        let mut this_rect = None;
        for event in window.events() {
            match event {
                Event::KeyReleased(_, Some('d'), _) => {
                    add_mode = AddMode::Draw;
                }
                Event::KeyReleased(_, Some('q'), _) => {
                    add_mode = AddMode::Query;
                }
                Event::KeyReleased(_, Some('l'), _) => {
                    query_mode = QueryMode::List;
                }
                Event::KeyReleased(_, Some('t'), _) => {
                    query_mode = QueryMode::Tree;
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

        match add_mode {
            AddMode::Draw => {
                if let Some(r) = this_rect {
                    rects.push(r);
                }
            }
            AddMode::Query => {
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

        draw_help_text(&mut frame, add_mode, query_mode);
    }

}
