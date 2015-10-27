use super::geom::Rect;

trait Spacial {
    fn aabb(&self) -> Rect;
}

struct QuadTreeConfig {
    max_children: usize,
    min_children: usize,
    max_depth: usize,
}

pub struct QuadTree<T> {
    root: QuadNode<T>,
    config: QuadTreeConfig
}

enum QuadNode<T> {
    Branch {
        aabb: Rect,
        children: [(Rect, Box<QuadNode<T>>); 4],
        element_count: usize,
        depth: usize,
    },
    Leaf {
        aabb: Rect,
        elements: Vec<T>,
        depth: usize,
    }
}

impl <T: Spacial> QuadTree<T> {
    pub fn insert(&mut self, t: T) {
        let &mut QuadTree { ref mut root, ref config } = self;
        root.insert(t, config);
    }
}

impl <T: Spacial> QuadNode<T> {
    fn new_leaf(aabb: Rect, depth: usize, config: &QuadTreeConfig) -> QuadNode<T> {
        QuadNode::Leaf {
            aabb: aabb,
            elements: Vec::with_capacity(config.max_children / 2),
            depth: depth
        }
    }

    fn insert(&mut self, t: T, config: &QuadTreeConfig) {
        let mut into = None;
        match self {
            &mut QuadNode::Branch { ref mut children, ref mut element_count, ..} => {
                for &mut (ref aabb, ref mut child) in children {
                    if aabb.contains(&t.aabb().top_left) {
                        child.insert(t, config);
                        break;
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
                    elements.push(t);
                }
            }
        }

        // If we transitioned from a leaf node to a branch node, we
        // need to update ourself and re-add all the children that we used to have
        // in our this leaf into our new leaves.
        if let Some((extracted_children, new_node)) = into {
            *self = new_node;
            for child in extracted_children {
                self.insert(child, config);
            }
        }
    }
}
