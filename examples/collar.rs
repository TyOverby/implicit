extern crate lux;
extern crate implicit;

mod helper;
mod display;

use implicit::*;
use implicit::geom::*;

// ALL
const DASH_SIZE_ON: f32 = 7.0;
const DASH_SIZE_OFF: f32 = 10.0;

// BASE
const NECK_CIRC: f32 = 1250.0;
const MAIN_HEIGHT: f32 = 100.0;

// FRONT
const FRONT_LEN: f32 = NECK_CIRC * (3.0 / 4.0);
const TRI_OFFSET: f32 = 50.0;

// BACK
const BACK_LEN: f32 = NECK_CIRC * (3.0 / 4.0);

// Holes
const HOLE_RADIUS: f32 = 12.5;
const HOLE_SPACING: f32 = HOLE_RADIUS * 4.0;
const HOLE_OFFSET: f32 = 50.0;
const STITCH_OFFSET: f32 = 10.0;
const NUM_HOLES: i32 = 5;

// Center
const CENTER_LEN: f32 = FRONT_LEN * (1.0 / 3.0);
const CENTER_HEIGHT: f32 = MAIN_HEIGHT * (1.0 + 1.0 / 4.0);
const CENTER_SHIFT_DOWN: f32 = MAIN_HEIGHT * (1.0 / 4.0);

// Hook Attach
const HOOK_CHUNK_HEIGHT: f32 = (CENTER_HEIGHT + CENTER_SHIFT_DOWN) / 4.0;
const HOOK_TOTAL_LEN: f32 = 300.0;
const HOOK_MIDDLE_LEN: f32 = 128.0;
const HOOK_ATTACH_SPACING: f32 = 51.5;
const HOOK_BASE_SPACING: f32 = 20.0;

fn front_outline() -> PolyGroup {
    let main_front_rect = Rectangle::new(Rect::from_points(
            &Point {x: 0.0, y: 0.0},
            &Point {x: FRONT_LEN, y: MAIN_HEIGHT}));

    let left_triangle = Polygon::new(vec![
            Point { x: -TRI_OFFSET, y: MAIN_HEIGHT / 2.0 },
            Point { x: 0.0, y: 0.0 },
            Point { x: 0.0, y: MAIN_HEIGHT }].into_iter());

    let right_triangle = Polygon::new(vec![
            Point { x: FRONT_LEN + TRI_OFFSET, y: MAIN_HEIGHT / 2.0 },
            Point { x: FRONT_LEN, y: 0.0 },
            Point { x: FRONT_LEN, y: MAIN_HEIGHT }].into_iter());


    let result = main_front_rect.or(left_triangle).or(right_triangle);

    result.fix_rules(2.25)
}

fn holes() -> Vec<Not<Circle>> {
    let mut holes = vec![];
    for i in 0 .. NUM_HOLES {
        holes.push(
            Circle {
                center: Point {
                    x: FRONT_LEN - HOLE_OFFSET - HOLE_SPACING * i as f32,
                    y: MAIN_HEIGHT / 2.0
                },
                radius: HOLE_RADIUS
            }.not());
        holes.push(
            Circle {
                center: Point {
                    x: HOLE_OFFSET + HOLE_SPACING * i as f32,
                    y: MAIN_HEIGHT / 2.0
                },
                radius: HOLE_RADIUS
            }.not());
    }
    holes
}

fn center() -> PolyGroup {
    let block = Rectangle::new(Rect::from_points(
            &Point {x: 0.0, y: 0.0},
            &Point {x: CENTER_LEN, y: CENTER_HEIGHT}));

    let left_triangle = Polygon::new(vec![
            Point { x: -TRI_OFFSET, y: CENTER_HEIGHT / 2.0 },
            Point { x: 0.0, y: 0.0 },
            Point { x: 0.0, y: CENTER_HEIGHT }].into_iter());

    let right_triangle = Polygon::new(vec![
            Point { x: CENTER_LEN + TRI_OFFSET, y: CENTER_HEIGHT / 2.0 },
            Point { x: CENTER_LEN, y: 0.0 },
            Point { x: CENTER_LEN, y: CENTER_HEIGHT }].into_iter());

    let result = block.or(left_triangle).or(right_triangle).translate(TRI_OFFSET, 0.0);
    result.fix_rules(2.25)
}

fn front() -> AndThese<SyncBox> {
    let front_outline = front_outline(); //.smooth(10.0, 5.0);
    let center = center();
    let center = center.center_at(&front_outline.center().unwrap());
    let center = center.translate(0.0, CENTER_SHIFT_DOWN);
    let front_collar = front_outline.or(center);//.smooth(20.0, 5.0);

    let mut targets: Vec<SyncBox> = holes().into_iter().map(|a| a.boxed()).collect();
    targets.push(front_collar.clone().boxed());

    AndThese { targets: targets }
}

fn back() -> SyncBox  {
    let back_outline = Rectangle::new(Rect::from_points(
            &Point {x: 0.0, y: 0.0},
            &Point {x: BACK_LEN, y: MAIN_HEIGHT}));
    let center = center();
    let center = center.center_at(&back_outline.center().unwrap());
    let center = center.translate(0.0, CENTER_SHIFT_DOWN);
    let back_collar = back_outline.or(center);
    back_collar.boxed()
}

fn hook_attach(middle_height: f32) -> Or<Or<Rectangle, Rectangle>, Rectangle> {
    let mid = Rectangle::new(Rect::from_point_and_size(
            &Point{x: (HOOK_TOTAL_LEN - HOOK_MIDDLE_LEN) / 2.0, y: HOOK_CHUNK_HEIGHT},
            &Vector{x: HOOK_MIDDLE_LEN, y: middle_height}));

    hook_attach_stitch(middle_height).or(mid)
}

fn hook_attach_stitch(middle_height: f32) -> Or<Rectangle, Rectangle> {
    let top = Rectangle::new(Rect::from_point_and_size(
            &Point{x: 0.0, y: 0.0},
            &Vector{x: HOOK_TOTAL_LEN, y: HOOK_CHUNK_HEIGHT}));

    let bot = Rectangle::new(Rect::from_point_and_size(
            &Point{x: 0.0, y: middle_height + HOOK_CHUNK_HEIGHT},
            &Vector{x: HOOK_TOTAL_LEN, y: HOOK_CHUNK_HEIGHT}));

    top.or(bot)
}


fn main() {
    fn offset(y: f32) -> Matrix {
        Matrix::new().translate(50.0, y)
    }

    let front_collar = front();
    let front_collar_outline = front_collar.clone().shrink(STITCH_OFFSET);
    let back_collar = back();
    let back_collar_outline = back_collar.clone().shrink(STITCH_OFFSET);

    let hook_attach_stitched = hook_attach_stitch(HOOK_BASE_SPACING).shrink(STITCH_OFFSET);
    let hook_attach_stitched = hook_attach_stitched.center_at(&front_collar.center().unwrap());

    let mut scene = Scene::new();
    scene.resolution = 0.5;

    let offset_50 = offset(0.0);
    let offset_250 = offset(200.0);
    let offset_450 = offset(400.0);
    let offset_650 = offset(600.0);
    let offset_850 = offset(800.0);
    let offset_850_right = offset_850.translate(400.0, 0.0);
    let center = front_collar.bounding_box().unwrap().midpoint();

    let f_c = scene.add_shape(&front_collar, RenderMode::Outline, offset_50);
    let f_o = scene.add_shape(&front_collar_outline, RenderMode::DashedPerfect(vec![DASH_SIZE_ON, DASH_SIZE_OFF]), offset_50);
    scene.add_shape(&hook_attach_stitched, RenderMode::DashedPerfect(vec![DASH_SIZE_ON, DASH_SIZE_OFF]), offset_50);

    scene.add_again(f_c, offset_250.mirror_horizontal(center.x));
    scene.add_again(f_o, offset_250.mirror_horizontal(center.x));

    let b_c = scene.add_shape(&back_collar, RenderMode::Outline, offset_450);
    let b_o = scene.add_shape(&back_collar_outline, RenderMode::DashedPerfect(vec![DASH_SIZE_ON, DASH_SIZE_OFF]), offset_450);
    scene.add_shape(&hook_attach_stitched, RenderMode::DashedPerfect(vec![DASH_SIZE_ON, DASH_SIZE_OFF]), offset_450);

    scene.add_again(b_c, offset_650.mirror_horizontal(center.x));
    scene.add_again(b_o, offset_650.mirror_horizontal(center.x));

    let hook_attach = hook_attach(HOOK_ATTACH_SPACING);
    let hook_attach_stitch = hook_attach_stitch(HOOK_ATTACH_SPACING).shrink(STITCH_OFFSET);

    let h_a = scene.add_shape(&hook_attach, RenderMode::Outline, offset_850);
    let h_a_s = scene.add_shape(&hook_attach_stitch, RenderMode::DashedPerfect(vec![DASH_SIZE_ON, DASH_SIZE_OFF]), offset_850);
    scene.add_again(h_a, offset_850_right);
    scene.add_again(h_a_s, offset_850_right);

    let mut pdf = PdfWriter::new("in", (1.0/100.0) * 72.0);
    scene.render_all(&mut pdf);
    pdf.write_out("collar.pdf");
}
