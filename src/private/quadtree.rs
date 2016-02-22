use std::collections::HashMap;
use std::cmp::Ord;
use super::geom::{Rect, Point, Line};

const EPSILON: f32 = 0.001;

pub trait Spatial {
    fn aabb(&self) -> Rect;
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct ItemId(u32);

#[derive(Debug, Clone)]
struct QuadTreeConfig {
    allow_duplicates: bool,
    max_children: usize,
    min_children: usize,
    max_depth: usize,
}

#[derive(Debug, Clone)]
pub struct QuadTree<T> {
    root: QuadNode,
    config: QuadTreeConfig,
    id: u32,
    elements: HashMap<ItemId, (T, Rect)>
}

#[derive(Debug)]
enum QuadNode {
    Branch {
        aabb: Rect,
        children: [(Rect, Box<QuadNode>); 4],
        in_all: Vec<(ItemId, Rect)>,
        element_count: usize,
        depth: usize,
    },
    Leaf {
        aabb: Rect,
        elements: Vec<(ItemId, Rect)>,
        depth: usize,
    }
}

impl Clone for QuadNode {
    fn clone(&self) -> QuadNode {
        match self {
            &QuadNode::Branch {ref aabb, ref children, ref in_all, ref element_count, ref depth} => {
                let children = [
                    children[0].clone(),
                    children[1].clone(),
                    children[2].clone(),
                    children[3].clone()];
                QuadNode::Branch{
                    aabb: aabb.clone(),
                    children: children,
                    in_all: in_all.clone(),
                    element_count: element_count.clone(),
                    depth: depth.clone(),
                }
            }
            &QuadNode::Leaf {ref aabb, ref elements, ref depth} => {
                QuadNode::Leaf {
                    aabb: aabb.clone(),
                    elements: elements.clone(),
                    depth: depth.clone()
                }
            }
        }
    }
}

impl <T> QuadTree<T> {
    pub fn new(size: Rect, allow_duplicates: bool,  min_children: usize, max_children: usize, max_depth: usize) -> QuadTree<T> {
        QuadTree {
            root: QuadNode::Leaf {
                aabb: size,
                elements: Vec::with_capacity(max_children),
                depth: 0,
            },
            config: QuadTreeConfig {
                allow_duplicates: allow_duplicates,
                max_children: max_children,
                min_children: min_children,
                max_depth: max_depth,
            },
            id: 0,
            elements: HashMap::with_capacity(max_children * 16)
        }
    }

    pub fn default(size: Rect) -> QuadTree<T> {
        QuadTree::new(size, true, 4, 16, 8)
    }

    pub fn insert_with_box(&mut self, t: T, aabb: Rect) -> ItemId {
        let &mut QuadTree { ref mut root, ref config, ref mut id, ref mut elements } = self;

        let item_id = ItemId(*id);
        *id += 1;

        elements.insert(item_id, (t, aabb));
        root.insert(item_id, aabb, config);
        item_id
    }

    pub fn first(&self) -> Option<ItemId> {
        self.elements.iter().next().map(|(id, _)| *id)
    }

    pub fn insert(&mut self, t: T) -> ItemId where T: Spatial {
        let b = t.aabb();
        self.insert_with_box(t, b)
    }

    pub fn query(&self, bounding_box: Rect) -> Vec<(&T, &Rect, ItemId)> where T: ::std::fmt::Debug {
        let mut ids = vec![];
        self.root.query(bounding_box, &mut ids);
        ids.sort_by(|&(id1, _), &(ref id2, _)| id1.cmp(id2));
        ids.dedup();
        ids.iter().map(|&(id, _)| {
            let &(ref t, ref rect) = match self.elements.get(&id) {
                Some(e) => e,
                None => {
                    panic!("looked for {:?}", id);
                }
            };
            (t, rect, id)
        }).collect()
    }

    pub fn remove(&mut self, item_id: ItemId) -> Option<(T, Rect)> {
        match self.elements.remove(&item_id) {
            Some((item, aabb)) => {
                self.root.remove(item_id, aabb, &self.config);
                Some((item, aabb))
            }
            None => None
        }
    }

    pub fn iter(&self) -> ::std::collections::hash_map::Iter<ItemId, (T, Rect)> {
        self.elements.iter()
    }

    pub fn inspect<F: FnMut(&Rect, usize, bool)>(&self, mut f: F) {
        self.root.inspect(&mut f);
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn validate(&self) -> bool where T: ::std::fmt::Debug {
        self.query(self.root.bounding_box());
        true
    }
}

impl QuadNode {
    fn bounding_box(&self) -> Rect {
        match self {
            &QuadNode::Branch {ref aabb, ..} => aabb.clone(),
            &QuadNode::Leaf {ref aabb, ..} => aabb.clone(),
        }
    }

    fn new_leaf(aabb: Rect, depth: usize, config: &QuadTreeConfig) -> QuadNode {
        QuadNode::Leaf {
            aabb: aabb,
            elements: Vec::with_capacity(config.max_children / 2),
            depth: depth
        }
    }

    fn inspect<F: FnMut(&Rect, usize, bool)>(&self, f: &mut F) {
        match self {
            &QuadNode::Branch { ref depth, ref aabb, ref children, .. } => {
                f(aabb, *depth, false);
                for child in children {
                    child.1.inspect(f);
                }
            }
            &QuadNode::Leaf { ref depth, ref aabb, .. } => {
                f(aabb, *depth, true);
            }
        }
    }

    fn insert(&mut self, item_id: ItemId, item_aabb: Rect, config: &QuadTreeConfig) {
        let mut into = None;
        match self {
            &mut QuadNode::Branch { ref aabb, ref mut in_all, ref mut children, ref mut element_count, .. } => {
                if item_aabb.contains(&aabb.midpoint()) {
                    // Only insert if there isn't another item with a very similar aabb.
                    if config.allow_duplicates || !in_all.iter().any(|&(_, ref e_bb)| e_bb.close_to(&item_aabb, EPSILON)) {
                        in_all.push((item_id, item_aabb));
                    } 
                } else {
                    for &mut (ref aabb, ref mut child) in children {
                        if aabb.does_intersect(&item_aabb) {
                            child.insert(item_id, item_aabb, config);
                        }
                    }
                }
                *element_count += 1;
            }

            &mut QuadNode::Leaf { ref aabb, ref mut elements, ref depth } => {
                if elements.len() == config.max_children && *depth != config.max_depth {
                    // STEAL ALL THE CHILDREN MUAHAHAHAHA
                    let mut extracted_children = Vec::new();
                    ::std::mem::swap(&mut extracted_children, elements);
                    extracted_children.push((item_id, item_aabb));

                    let split = aabb.split_quad();
                    into = Some((extracted_children, QuadNode::Branch {
                        aabb: *aabb,
                        in_all: Vec::new(),
                        children: [
                            (split[0], Box::new(QuadNode::new_leaf(split[0], depth + 1, config))),
                            (split[1], Box::new(QuadNode::new_leaf(split[1], depth + 1, config))),
                            (split[2], Box::new(QuadNode::new_leaf(split[2], depth + 1, config))),
                            (split[3], Box::new(QuadNode::new_leaf(split[3], depth + 1, config))),
                        ],
                        element_count: 0,
                        depth: *depth
                    }));
                } else {
                    if config.allow_duplicates || !elements.iter().any(|&(_, ref e_bb)| e_bb.close_to(&item_aabb, EPSILON)) {
                        elements.push((item_id, item_aabb));
                    } 
                }
            }
        }

        // If we transitioned from a leaf node to a branch node, we
        // need to update ourself and re-add all the children that we used to have
        // in our this leaf into our new leaves.
        if let Some((extracted_children, new_node)) = into {
            *self = new_node;
            for (child_id, child_aabb) in extracted_children {
                self.insert(child_id, child_aabb, config);
            }
        }
    }

    fn remove(&mut self, item_id: ItemId, item_aabb: Rect, config: &QuadTreeConfig) -> bool {
        fn remove_from(v: &mut Vec<(ItemId, Rect)>, item: ItemId) -> bool {
            if let Some(index) = v.iter().position(|a| a.0 == item) {
                v.swap_remove(index);
                true
            } else {
                false
            }
        }

        let mut compact = None;
        let removed = match self {
            &mut QuadNode::Branch { ref depth, ref aabb, ref mut in_all, ref mut children, ref mut element_count, .. } => {
                let mut did_remove = false;

                if item_aabb.contains(&aabb.midpoint()) {
                    did_remove = remove_from(in_all, item_id);
                } else {
                    for &mut (ref child_aabb, ref mut child_tree) in children {
                        if child_aabb.does_intersect(&item_aabb) {
                            did_remove |= child_tree.remove(item_id, item_aabb, config);
                        }
                    }
                }

                if did_remove {
                    *element_count -= 1;
                    if *element_count < config.min_children {
                        compact = Some((*element_count, *aabb, *depth));
                    }
                }
                did_remove
            }

            &mut QuadNode::Leaf { ref mut elements, ..} => remove_from(elements, item_id)
        };

        if let Some((size, aabb, depth)) = compact {
            let mut elements = Vec::with_capacity(size);
            self.query(aabb, &mut elements);
            elements.sort_by(|&(id1, _), &(ref id2, _)| id1.cmp(id2));
            elements.dedup();
            *self = QuadNode::Leaf {
                aabb: aabb,
                elements: elements,
                depth: depth
            };
        }
        removed
    }

    fn query(&self, query_aabb: Rect, out: &mut Vec<(ItemId, Rect)>) {
        fn match_all(elements: &Vec<(ItemId, Rect)>, query_aabb: Rect, out: &mut Vec<(ItemId, Rect)>) {
            for &(ref child_id, ref child_aabb) in elements {
                if query_aabb.does_intersect(child_aabb) {
                    out.push((*child_id, *child_aabb))
                }
            }
        }

        match self {
            &QuadNode::Branch { ref in_all, ref children, .. } => {
                match_all(in_all, query_aabb, out);

                for &(ref child_aabb, ref child_tree) in children {
                    if query_aabb.does_intersect(&child_aabb) {
                        child_tree.query(query_aabb, out);
                    }
                }
            }
            &QuadNode::Leaf { ref elements, .. } =>
                match_all(elements, query_aabb, out)
        }
    }
}

impl Spatial for Rect {
    fn aabb(&self) -> Rect {
        *self
    }
}

impl Spatial for Point {
    fn aabb(&self) -> Rect {
        Rect::null_at(self)
    }
}

impl Spatial for Line {
    fn aabb(&self) -> Rect {
        Rect::from_points(&self.0, &self.1)
    }
}
