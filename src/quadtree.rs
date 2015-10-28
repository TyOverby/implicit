use std::collections::HashMap;
use super::geom::Rect;

pub trait Spatial {
    fn aabb(&self) -> Rect;
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct ItemId(u32);

struct QuadTreeConfig {
    max_children: usize,
    min_children: usize,
    max_depth: usize,
}

pub struct QuadTree<T> {
    root: QuadNode,
    config: QuadTreeConfig,
    id: u32,
    elements: HashMap<ItemId, (T, Rect)>
}

enum QuadNode {
    Branch {
        aabb: Rect,
        children: [(Rect, Box<QuadNode>); 4],
        element_count: usize,
        depth: usize,
    },
    Leaf {
        aabb: Rect,
        elements: Vec<(ItemId, Rect)>,
        depth: usize,
    }
}

impl <T> QuadTree<T> {
    pub fn insert_with_box(&mut self, t: T, aabb: Rect) -> ItemId {
        let &mut QuadTree { ref mut root, ref config, ref mut id, ref mut elements } = self;

        let item_id = ItemId(*id);
        *id += 1;

        elements.insert(item_id, (t, aabb));
        root.insert(item_id, aabb, config);
        item_id
    }

    pub fn insert(&mut self, t: T) -> ItemId where T: Spatial {
        let b = t.aabb();
        self.insert_with_box(t, b)
    }

    pub fn query(&self, bounding_box: Rect) -> Vec<&(T, Rect)> {
        let mut ids = vec![];
        self.root.query(bounding_box, &mut ids);
        ids.iter().map(|&(id, _)| self.elements.get(&id).unwrap()).collect()
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
}

impl QuadNode {
    fn new_leaf(aabb: Rect, depth: usize, config: &QuadTreeConfig) -> QuadNode {
        QuadNode::Leaf {
            aabb: aabb,
            elements: Vec::with_capacity(config.max_children / 2),
            depth: depth
        }
    }

    fn insert(&mut self, item_id: ItemId, item_aabb: Rect, config: &QuadTreeConfig) {
        let mut into = None;
        match self {
            &mut QuadNode::Branch { ref mut children, ref mut element_count, .. } => {
                for &mut (ref aabb, ref mut child) in children {
                    if aabb.does_intersect(&item_aabb) {
                        child.insert(item_id, item_aabb, config);
                    }
                }
                *element_count += 1;
            }

            &mut QuadNode::Leaf { ref aabb, ref mut elements, ref depth } => {
                if elements.len() == config.max_children && *depth != config.max_depth {
                    // STEAL ALL THE CHILDREN MUAHAHAHAHA
                    let mut extracted_children = Vec::new();
                    ::std::mem::swap(&mut extracted_children, elements);

                    let split = aabb.split_quad();
                    into = Some((extracted_children, QuadNode::Branch {
                        aabb: *aabb,
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
                    elements.push((item_id, item_aabb));
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
        let mut compact = None;
        let removed = match self {
            &mut QuadNode::Branch { ref depth, ref aabb, ref mut children, ref mut element_count, .. } => {
                let mut did_remove = false;
                for &mut (ref child_aabb, ref mut child_tree) in children {
                    if child_aabb.does_intersect(&item_aabb) {
                        did_remove |= child_tree.remove(item_id, item_aabb, config);
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

            &mut QuadNode::Leaf { ref mut elements, ..} => {
                let len_before = elements.len();
                elements.retain(|a| a.0 == item_id);
                let len_after = elements.len();
                len_before != len_after
            }
        };

        if let Some((size, aabb, depth)) = compact {
            let mut elements = Vec::with_capacity(size);
            self.query(aabb, &mut elements);
            *self = QuadNode::Leaf {
                aabb: aabb,
                elements: elements,
                depth: depth
            };
        }
        removed
    }

    fn query(&self, aabb: Rect, out: &mut Vec<(ItemId, Rect)>) {
        match self {
            &QuadNode::Branch { ref children, .. } => {
                for &(ref child_aabb, ref child_tree) in children {
                    if aabb.does_intersect(&child_aabb) {
                        child_tree.query(aabb, out);
                    }
                }
            }
            &QuadNode::Leaf { ref elements, .. } => {
                for &(ref child_id, ref child_aabb) in elements {
                    if aabb.does_intersect(child_aabb) {
                        out.push((*child_id, *child_aabb))
                    }
                }
            }
        }
    }
}
