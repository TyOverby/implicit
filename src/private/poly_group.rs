use ::geom::{Polygon, Line, Rect, Point, Vector};
use ::Implicit;
use ::{Spatial, QuadTree};

pub struct PolyGroup;


struct PolySegment {
    line: Line,
    additive: bool,
}

struct Grid(Vec<Vec<Bucket>>);

struct Bucket(Rect, Vec<Line>);

impl Spatial for PolySegment {
    fn aabb(&self) -> Rect {
        self.line.aabb()
    }
}

impl PolyGroup {
    pub fn new(polys: Vec<Polygon>) -> PolyGroup {
        let (additive, subtractive, bb) = PolyGroup::segregate_polys(polys);
        let qt = PolyGroup::build_qt(additive, subtractive, bb);
        PolyGroup
    }

    /// Builds a grid containing the nearest groups of lines
    fn build_grid(qt: QuadTree<PolySegment>, divisions: usize, bb: Rect) -> Grid {
        let dx = bb.width() / divisions as f32;
        let dy = bb.width() / divisions as f32;

        for x in 0 .. divisions {
            let px = dx * x as f32;
            for y in 0 .. divisions {
                let py = dy * y as f32;
                let rect = Rect::from_point_and_size(&Point{x: px, y: py}, &Vector{x: dx, y: dy});
            }
        }
        unimplemented!();
    }

    /// Builds a quad tree out of the additive and subtractive polygons
    fn build_qt(additive: Vec<Polygon>, subtractive: Vec<Polygon>, bb: Rect) -> QuadTree<PolySegment> {
        let mut qt = QuadTree::default(bb);
        for poly in &additive {
            for line in poly.lines().iter().cloned() {
                qt.insert(PolySegment { line: line, additive: true });
            }
        }

        for poly in &subtractive {
            for line in poly.lines().iter().cloned() {
                qt.insert(PolySegment { line: line, additive: false });
            }
        }
        qt
    }

    /// Find out which polygons are additive and which are subtractive
    fn segregate_polys(mut v: Vec<Polygon>) -> (Vec<Polygon>, Vec<Polygon>, Rect) {
        let mut additive = vec![];
        let mut subtractive = vec![];
        let mut bounding_box = Rect::null();
        while let Some(p) = v.pop() {
            let mut inside_count = 0;
            for test in v.iter().chain(&additive).chain(&subtractive) {
                let first_point = p.points()[0];
                if test.sample(first_point) < 0.0 {
                    inside_count += 1;
                }
            }
            bounding_box = bounding_box.union_with(&p.bounding_box().unwrap());
            if inside_count % 2 == 0 {
                additive.push(p);
            } else {
                subtractive.push(p);
            }
        }

        (additive, subtractive, bounding_box)
    }
}
