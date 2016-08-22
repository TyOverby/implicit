use super::{
    Point,
    Rect,
    Bitmap,
};
use ::{Implicit, QuadTree};

#[derive(Copy, Clone, Debug)]
struct SampleDist {
    pub x_bump: f32,
    pub y_bump: f32,
}

impl SampleDist {
    fn modify_bb(&self, bb: &mut Rect) {
        let top_left = {
            let Point{ x, y } = bb.top_left();
            let (x, y) = self.floor(x, y);
            Point{x: x, y: y}
        };
        let bottom_right = {
            let Point { x, y } = bb.bottom_right();
            let (x, y) = self.floor(x, y);
            Point{x: x, y: y}
        };

        *bb = Rect::from_points(&top_left, &bottom_right);
    }
    fn floor(&self, x: f32, y: f32) -> (f32, f32){
        let x = x - (x % self.x_bump);
        let y = y - (y % self.y_bump);
        (x, y)
    }
    fn bump_x(&self, x: f32) -> f32 {
        x + self.x_bump
    }
    fn bump_y(&self, x: f32) -> f32 {
        x + self.x_bump
    }
    fn max_bump(&self) -> f32 {
        self.x_bump.max(self.y_bump)
    }
}


pub fn sampling_points<S: Implicit>(shape: &S, resolution: f32) -> Vec<Point> {
    fn sample_with_bitmap(rect: Rect, sample_dist: SampleDist, bitmap: &Bitmap) -> (Vec<Point>, Vec<Point>) {
        fn real_sample_with_bitmap(rect: Rect, sample_dist: SampleDist, bitmap: &Bitmap) -> (Vec<Point>, Vec<Point>) {
            let mut out_uncontested = vec![];
            let mut out_contested = vec![];
            let cmp = sample_dist.max_bump() * 5.0;

            for (p, c) in sample_from_box(rect, sample_dist) {
                let Point{x, y} = p;
                let sample = bitmap.sample(x, y, |a, b, c, d| (a.abs()).min(b.abs()).min(c.abs()).min(d.abs()));

                let bll = sample < cmp;
                match (bll, c) {
                    (true, true) => {
                        out_uncontested.push(p);
                    }
                    (true, false) =>  {
                        out_contested.push(p);
                    }
                    (false, _) => {}
                }
            }

            (out_uncontested, out_contested)
        }


        fn parallel(rect: Rect, sample_dist: SampleDist, bitmap: &Bitmap, thread_count: u32, target_threads: u32, main_thread_id: usize) -> (Vec<Point>, Vec<Point>) {
            if thread_count >= target_threads {
                let r = ::flame::span_of("real sample with bitmap", || real_sample_with_bitmap(rect, sample_dist, bitmap));
                if ::thread_id::get() != main_thread_id {
                    ::flame::commit_thread();
                }
                return r;
            }

            let (top, bot) = rect.split_hori();
            let ((mut u1, mut c1), (mut u2, mut c2)) =
                ::rayon::join(move || parallel(top, sample_dist, bitmap, thread_count * 2, target_threads, main_thread_id),
                              move || parallel(bot, sample_dist, bitmap, thread_count * 2, target_threads, main_thread_id));
            u1.append(&mut u2);
            c1.append(&mut c2);

            if ::thread_id::get() != main_thread_id {
                ::flame::commit_thread();
            }
            (u1, c1)
        }

        parallel(rect, sample_dist, bitmap, 1, 7, ::thread_id::get())
    }

    let bb = shape.bounding_box().unwrap();
    let expand = resolution * 2.0;
    let bb = bb.expand(expand, expand, expand, expand);
    let sample_dist = SampleDist { x_bump: resolution, y_bump: resolution };

    ::flame::start("build poor mans quad tree");
    //let (pmqt, _) = PmQuadTree::build(shape, bb, sample_dist).unwrap();
    ::flame::end("build poor mans quad tree");

    ::flame::start("build bitmap");
    let bitmap = Bitmap::new(shape, sample_dist.max_bump() * 5.0);
    ::flame::end("build bitmap");

    ::flame::start("filter sample with bitmap");
    let (mut out, to_filter) = sample_with_bitmap(bb, sample_dist, &bitmap);
    ::flame::end("filter sample with bitmap");

    ::flame::start("filter points");
    let mut quadtree = QuadTree::new(bb, false,  5, 20, 5);
    for contested in to_filter {
        quadtree.insert(contested);
    }
    for (_, &(ok, _)) in quadtree.iter() {
        out.push(ok);
    }
    ::flame::end("filter points");

    out
}

fn sample_from_box(mut bb: Rect, sample_dist: SampleDist) -> BoxSampler {
    sample_dist.modify_bb(&mut bb);
    let Point{x, y} = bb.top_left();
    let x_orig = x;
    BoxSampler {
        x: x,
        y: y,
        bb: bb,
        x_orig: x_orig,
        sample_dist: sample_dist,
        on_top: true,
    }
}

struct BoxSampler {
    x: f32,
    y: f32,
    bb: Rect,
    x_orig: f32,
    sample_dist: SampleDist,
    on_top: bool,
}

impl Iterator for BoxSampler {
    type Item = (Point, bool);
    fn next(&mut self) -> Option<(Point, bool)> {
        let p = Point{x: self.x, y: self.y};
        let on_left = p.x == self.x_orig;
        let on_right = self.sample_dist.bump_x(self.x) >= self.bb.right();
        let on_top = self.on_top;
        let on_bottom = self.sample_dist.bump_y(self.y) >= self.bb.bottom();

        if self.bb.contains(&p) {
            self.x = self.sample_dist.bump_x(self.x);
            Some((p, !(on_left || on_right || on_top || on_bottom)))
        } else {
            self.x = self.x_orig;
            self.y = self.sample_dist.bump_y(self.y);
            self.on_top = false;
            if !self.bb.contains(&Point{x: self.x, y: self.y}) {
                None
            } else {
                self.next()
            }
        }
    }
}

