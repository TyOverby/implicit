extern crate lux;
extern crate implicit;

mod helper;
mod display;

use implicit::*;
use implicit::formats::pdf::*;
use implicit::geom::*;

const WIDTH: f32 = 3.37;
const HEIGHT: f32 = 2.12;
const DEPTH: f32 = 0.50;

fn panels() -> OrThese<Rectangle> {
    let flap = Rectangle::new(Rect::from_point_and_size(
            &Point{ x: 0.0, y: 0.0 },
            &Vector{ x: WIDTH, y: HEIGHT }
    ));
    let back = Rectangle::new(Rect::from_point_and_size(
            &Point{x: 0.0, y: HEIGHT + DEPTH},
            &Vector{x: WIDTH, y: HEIGHT}
    ));
    let front = Rectangle::new(Rect::from_point_and_size(
            &Point{x: 0.0, y: 2.0 * (HEIGHT + DEPTH)},
            &Vector{x: WIDTH, y: HEIGHT}
    ));
    let left = Rectangle::new(Rect::from_point_and_size(
            &Point{
                x: -1.0 * (DEPTH + WIDTH / 2.0),
                y: 2.0 * (HEIGHT + DEPTH)},
            &Vector{
                x: WIDTH / 2.0,
                y: HEIGHT
            }
    ));
    let right = Rectangle::new(Rect::from_point_and_size(
            &Point{
                x: WIDTH + DEPTH,
                y: 2.0 * (HEIGHT + DEPTH)},
            &Vector{
                x: WIDTH / 2.0,
                y: HEIGHT
            }
    ));

    OrThese::new(vec![flap, back, front, left, right])
}

fn gaps() -> OrThese<Rectangle> {
    let flap_back = Rectangle::new(Rect::from_point_and_size(
            &Point{x: 0.0, y: HEIGHT},
            &Vector{x: WIDTH, y: DEPTH}
    ));
    let back_front = Rectangle::new(Rect::from_point_and_size(
            &Point{x: 0.0, y: 2.0 * HEIGHT + DEPTH},
            &Vector{x: WIDTH, y: DEPTH}
    ));
    let left_front = Rectangle::new(Rect::from_point_and_size(
            &Point{x: -DEPTH, y: 2.0 * (HEIGHT + DEPTH)},
            &Vector{x: DEPTH, y: HEIGHT}
    ));
    let right_front = Rectangle::new(Rect::from_point_and_size(
            &Point{x: WIDTH, y: 2.0 * (HEIGHT + DEPTH)},
            &Vector{x: DEPTH, y: HEIGHT}
    ));
    OrThese::new(vec![flap_back, back_front, left_front, right_front])
}

fn main() {
    let mut scene = Scene::new();
    scene.recursion_depth = 8;

    let panels = panels();
    let gaps = gaps();
    let panels = panels.or(gaps.clone());


    scene.add_shape(&panels, RenderMode::Outline, Matrix::new().translate(WIDTH / 2.0 + DEPTH, 0.0));
    scene.add_shape(&gaps, RenderMode::BasicDashed(vec![0.5, 0.5]), Matrix::new().translate(WIDTH / 2.0 + DEPTH, 0.0));

    let mut pdf = PdfWriter::new("in", 1.0 * 72.0);
    scene.render_all(&mut pdf);
    pdf.write_out("pouch.pdf");

    helper::display(&[&panels]);
}
