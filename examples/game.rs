extern crate lux;
#[macro_use]
extern crate implicit;
extern crate num_traits;

mod helper;

use implicit::*;
use implicit::formats::pdf::PdfWriter;
use implicit::geom::*;

const SIZE: f32 = 1.0;

fn hex(x: f32, y: f32) -> Polygon {
    fn corner(center: (f32, f32), i: u32) -> Point {
        use std::f32::consts::PI;
        let i = i as f32;
        let angle_deg = 60.0 * i + 30.0;
        let angle_rad = PI / 180.0 * angle_deg;
        Point {
            x: center.0 + SIZE * angle_rad.cos(),
            y: center.1 + SIZE * angle_rad.sin(),
        }
    }

    let center = (x, y);
    Polygon::new(vec![
        corner(center, 0),
        corner(center, 1),
        corner(center, 2),
        corner(center, 3),
        corner(center, 4),
        corner(center, 5),
    ].into_iter())
}

fn width() -> f32 {
    use num_traits::Float;

    (3.0).sqrt()
}
fn height() -> f32 {
    2.0
}

fn grid() -> Vec<Polygon> {
    fn row(offset: u32, h: f32, n: u32) -> Vec<Polygon> {
        let offset = offset as f32 * width();
        let mut out = vec![];
        let jagged_offset = if n % 2 == 0 { 0.0 } else { - width() * 0.5 };

        for i in 0 .. n {
            let x = offset + jagged_offset + i as f32 * width();
            out.push(hex(x, h));
        }

        out
    }

    let mut out = vec![];
    for (i, count) in vec![5, 6, 7, 8, 9, 8, 7, 6, 5].into_iter().enumerate() {
        out.append(&mut row((9 - count) / 2, i as f32 * 0.75 * height(), count));
    }

    out
}

fn main() {
    fn write_out<F: ApplyFigure + 'static>(name: &str, figure: F) {
        let mut scene = Scene::new();
        scene.recursion_depth = 10;
        scene.add(figure);

        let mut pdf = PdfWriter::new("in", (1.0/200.0) * 72.0);
        scene.render_all(&mut pdf);
        pdf.write_out(name);
    }

    let gridded = OrThese::new(grid().into_iter().map(|s| {
        s.shrink(0.1).scale(100.0).smooth(25.0, 10)
    }).collect());

    let backing = OrThese::new(grid().into_iter().map(|s| {
        s.grow(0.4).scale(100.0).smooth(0.0, 10)
    }).collect());

    write_out("pieces.pdf", figure!((gridded.clone())));
    write_out("backing.pdf", figure!((backing.clone())));
    write_out("border.pdf", figure!((backing.and_not(gridded))));
}
