extern crate implicit;
extern crate lux;
extern crate rand;

use lux::prelude::*;
use lux::color;
use lux::interactive::Event;

use rand::{thread_rng, Rng};

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

fn draw_tree_query(rects: &[Rect], query: Option<&Rect>, frame: &mut Frame) {
    let size = Rect::from_point_and_size(
        &Point {
            x: 0.0,
            y: 0.0
        },
        &Vector {
            x: frame.width(),
            y: frame.height()
        });

    let mut quadtree = QuadTree::default(size);

    for rect in rects {
        quadtree.insert(*rect);
    }

    for (_, &(ref rect, _)) in quadtree.iter() {
        draw_rectangle(frame, rect, (0.0, 0.0, 0.0));
    }

    if let Some(query) = query {
        for &(ref rect, _) in quadtree.query(*query) {
            draw_rectangle(frame, rect, (0.0, 0.0, 0.5));
        }
    }

    quadtree.inspect(|rect, _, _| {
        draw_rectangle(frame, rect, (0.5, 0.0, 0.0));
    });

}

fn draw_list_query(rects: &[Rect], query: Option<&Rect>, frame: &mut Frame) {
    for rect in rects {
        if let Some(query) = query {
            if query.does_intersect(rect) {
                draw_rectangle(frame, rect, (0.0, 0.0, 0.5));
            } else {
                draw_rectangle(frame, rect, (0.0, 0.0, 0.0));
            }
        } else {
            draw_rectangle(frame, rect, (0.0, 0.0, 0.0));
        }
    }
}

fn main() {
    let mut window = Window::new_with_defaults().unwrap();
    let mut last_down_position = None;
    let mut rects = vec![];
    let mut add_mode = AddMode::Draw;
    let mut query_mode = QueryMode::List;
    let mut query = None;

    for _ in 0 .. 1_000 {
        let x1 = thread_rng().gen::<f32>() * window.width();
        let y1 = thread_rng().gen::<f32>() * window.height();
        let x2 = thread_rng().gen::<f32>() * window.width() / 10.0;
        let y2 = thread_rng().gen::<f32>() * window.height() / 10.0;

        let p1 = Point { x: x1, y: y1 };
        let v = Vector { x: x2, y: y2 };
        let p2 = p1 + v;

        let rect = Rect::from_points(&p1, &p2);
        rects.push(rect);
    }

    while window.is_open() {
        let mut frame = window.cleared_frame(color::WHITE);
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

        match (add_mode, this_rect) {
            (AddMode::Draw, Some(r)) => {
                rects.push(r);
            }
            (AddMode::Query, Some(r)) => {
                query = Some(r);
            }
            _ => {  }
        }

        match query_mode {
            QueryMode::List => draw_list_query(&rects[..], query.as_ref(), &mut frame),
            QueryMode::Tree => draw_tree_query(&rects[..], query.as_ref(), &mut frame),
        }

        if let Some(query) = query.as_ref() {
            draw_rectangle(&mut frame, query, (0.0, 0.5, 0.0));
        }

        if let Some(last_down) = last_down_position {
            let rect = Rect::from_points(
                &Point { x: last_down.0, y: last_down.1 },
                &Point { x: cur_pos.0, y: cur_pos.1 }
            );
            draw_rectangle(&mut frame, &rect, (0.8, 0.8, 0.8));
        }

        draw_help_text(&mut frame, add_mode, query_mode);
    }
}
