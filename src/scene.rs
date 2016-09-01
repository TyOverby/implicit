use ::{OutputMode, RenderMode, Implicit, render, OutputDevice};
use ::util::geom::{Point, Rect, Matrix};

#[macro_export]
macro_rules! figure {
    () => { () };
    (($shape:expr, $mode:expr, $matrix:expr, $mask:expr)) => {
        FigureLink {
            shape: $shape,
            mode: $mode,
            matrix: $matrix,
            mask: $mask,
            next: (),
        }
    };
    (($shape:expr, $mode:expr, $matrix:expr, $mask:expr), $($rest:tt),+) => {
        FigureLink {
            shape: $shape,
            mode: $mode,
            matrix: $matrix,
            mask: $mask,
            next: figure!($($rest),+),
        }
    };

    (($shape:expr, $mode:expr, $matrix:expr)) => {
        figure![($shape, $mode, $matrix, None)]
    };
    (($shape:expr, $mode:expr, $matrix:expr), $($rest:tt),+) => {
        figure![($shape, $mode, $matrix, None), $($rest),+]
    };
    (($shape:expr, $mode:expr)) => {
        figure![($shape, $mode, None, None)]
    };
    (($shape:expr, $mode:expr), $($rest:tt),+) => {
        figure![($shape, $mode, None, None), $($rest),+]
    };
    (($shape:expr)) => {
        figure![($shape, $crate::RenderMode::Outline, None, None)]
    };
    (($shape:expr), $($rest:tt),+) => {
        figure![($shape, $crate::RenderMode::Outline, None, None), $($rest),+]
    };
}

pub trait ApplyFigure {
    fn analyze(&self, state: &mut FigureState);
    fn render(&self, state: &mut FigureState);
}

pub struct FigureState {
    low_x: f32,
    low_y: f32,
    figure_bb: Rect,
    adjusted_bb: Rect,
    shapes: Vec<OutputMode>,

    current_y: f32,
    recursion_depth: u32,
}

pub struct FigureLink<'a, S: Implicit + 'a, N: ApplyFigure> {
    pub shape: &'a S,
    pub mask: Option<&'a Implicit>,
    pub matrix: Option<Matrix>,
    pub mode: RenderMode<'a>,

    pub next: N
}

impl ApplyFigure for () {
    fn analyze(&self, _: &mut FigureState) { }
    fn render(&self, _: &mut FigureState) { }
}

impl <'a, S: Implicit + Sync, N: ApplyFigure> ApplyFigure for FigureLink<'a, S, N> {

    fn analyze(&self, state: &mut FigureState) {
        let bb = self.shape.bounding_box().unwrap();
        let bb = transform_bounding_box(bb, self.matrix.unwrap_or(Matrix::new()));
        let Point{x, y} = bb.top_left();

        state.figure_bb = state.figure_bb.union_with(&bb);
        state.low_x = state.low_x.min(x);
        state.low_y = state.low_y.min(y);

        // Continue
        self.next.analyze(state);
    }

    fn render(&self, state: &mut FigureState) {
        let shape = self.shape.translate(-state.low_x, state.current_y - state.low_y);
        let bb = shape.bounding_box().unwrap();
        let bb = transform_bounding_box(bb, self.matrix.unwrap_or(Matrix::new()));
        state.adjusted_bb = state.adjusted_bb.union_with(&bb);

        let out = render(shape, &self.mode, state.recursion_depth, true);
        state.shapes.push(out);

        // Continue
        self.next.render(state);
    }
}



pub struct Scene {
    shapes: Vec<OutputMode>,
    pub recursion_depth: u32,
    pub total_bounding_box: Rect,
    current_y: f32,
    margin: f32,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            shapes: Vec::new(),
            current_y: 0.0,
            recursion_depth: 8,
            total_bounding_box: Rect::null_at(&Point{ x: 0.0, y: 0.0}),
            margin: 10.0,
        }
    }

    pub fn add<L: ApplyFigure>(&mut self, list: L) {
        let null_rect = Rect::null();

        let mut state = FigureState {
            low_x: ::std::f32::INFINITY,
            low_y: ::std::f32::INFINITY,
            figure_bb: null_rect,
            adjusted_bb: null_rect,
            shapes: vec![],

            current_y: self.current_y,
            recursion_depth: self.recursion_depth,
        };

        list.analyze(&mut state);
        list.render(&mut state);

        self.shapes.append(&mut state.shapes);
        self.current_y += state.figure_bb.height();
        self.current_y += self.margin;
        self.total_bounding_box = self.total_bounding_box.union_with(&state.adjusted_bb);
    }


    pub fn render_all<O: OutputDevice>(&self, out: &mut O) {
        out.set_size(self.total_bounding_box.width(), self.total_bounding_box.height());
        for rendered in &self.shapes {
            match rendered {
                &OutputMode::Solid(_) => unimplemented!(),
                &OutputMode::Outline(ref lines) => {
                    for line in lines {
                        out.start_line();
                        let start = line[0];
                        for p in line { out.add_point(*p); }
                        out.add_point(start);
                        out.end_line();
                    }
                },
                &OutputMode::DashedLine(ref dashed) => {
                    for dashed_line in dashed {
                        for segment in dashed_line.segments() {
                            out.start_line();
                            for p in segment {
                                out.add_point(*p);
                            }
                            out.end_line();
                        }
                    }
                }
            }
        }
    }
}

fn transform_bounding_box(bb: Rect, matrix: Matrix) -> Rect {
    let a = matrix.transform_point(&bb.top_left());
    let b = matrix.transform_point(&bb.top_right());
    let c = matrix.transform_point(&bb.bottom_left());
    let d = matrix.transform_point(&bb.bottom_right());
    let mut new_bb = Rect::null_at(&a);
    new_bb.expand_to_include(&b);
    new_bb.expand_to_include(&c);
    new_bb.expand_to_include(&d);
    new_bb
}
