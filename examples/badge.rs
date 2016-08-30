extern crate lux;
extern crate implicit;
extern crate flame;

mod helper;
mod display;

use implicit::*;
use implicit::geom::*;

const CARD_WIDTH:  f32 = 300.0;
const CARD_HEIGHT: f32 = 500.0;
const CURF: f32 = 100.0;

fn main() {
    let card_size = Rect::from_point_and_size(&Point{x: 0.0, y: 0.0}, &Vector{x: CARD_WIDTH, y: CARD_HEIGHT});
    let card_size_expanded = card_size.expand(CURF, CURF, CURF, CURF);

    let card_outline = Rectangle::new(card_size);
    let backing = Rectangle::new(card_size_expanded);
    let cutout = card_outline.shrink(20.0);

    let cutout = cutout.smooth(20.0, 8);
    let backing = backing.smooth(20.0, 8);

    let front = backing.and_not(cutout);

    helper::display(&[&front]);
}
