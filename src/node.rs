use replace_with::replace_with_or_abort;
pub(super) const BLOCK_SIZE: usize = 1000;
pub(super) const LEFT: usize = 0;
pub(super) const RIGHT: usize = 1;

pub(super) enum Node<C> {
    Leaf(Vec<C>),
    Inner(usize, [Box<Node<C>>; 2]),
}

impl<C> Node<C> {
    pub fn rotate(&mut self, direction: usize) {
        // pic for rotating left
        //     self        --->    n2
        //  n1       n2       self     n4
        //          n3 n4    n1   n3
        replace_with_or_abort(self, |owned_self| {
            let [n1, n2] = owned_self.extract_children(direction);
            let [n3, n4] = n2.extract_children(direction);
            let new_self_size = n1.len() + n3.len();
            let new_self = Box::new(Node::Inner(new_self_size, [n1, n3]));
            let new_n2_size = new_self_size + n4.len();
            let new_n2 = Node::Inner(new_n2_size, [new_self, n4]);
            new_n2
        })
    }
    pub fn extract_children(self, direction: usize) -> [Box<Node<C>>; 2] {
        let mut children = match self {
            Node::Leaf(_) => panic!("extracting children from a leaf"),
            Node::Inner(_, children) => children,
        };
        if direction == RIGHT {
            children.swap(0, 1)
        }
        children
    }
    pub fn insert(&mut self, index: usize, element: C) -> &mut C {
        if self.is_leaf() && self.len() == BLOCK_SIZE {
            self.divide()
        }
        match self {
            Node::Leaf(block) => {
                block.insert(index, element);
                &mut block[index]
            }
            Node::Inner(size, children) => {
                *size += 1;
                let left_size = children[LEFT].len();
                if left_size >= index {
                    children[LEFT].insert(index, element)
                } else {
                    children[RIGHT].insert(index - left_size, element)
                }
            }
        }
    }
    pub fn divide(&mut self) {
        replace_with_or_abort(self, |owned_self| {
            let mut block: Vec<C> = match owned_self {
                Node::Leaf(inner_block) => inner_block,
                _ => unreachable!(),
            };
            let size = block.len();
            let right_block = block.split_off(size / 2);
            Node::Inner(
                size,
                [
                    Box::new(Node::Leaf(block)),
                    Box::new(Node::Leaf(right_block)),
                ],
            )
        });
    }
    pub fn is_leaf(&self) -> bool {
        match self {
            Node::Leaf(_) => true,
            _ => false,
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Node::Leaf(block) => block.len(),
            Node::Inner(size, _) => *size,
        }
    }
    pub fn get(&self, index: usize) -> Option<&C> {
        match self {
            Node::Leaf(block) => block.get(index),
            Node::Inner(_, children) => {
                let left_size = children[LEFT].len();
                if left_size >= index {
                    children[LEFT].get(index)
                } else {
                    children[RIGHT].get(index - left_size)
                }
            }
        }
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut C> {
        match self {
            Node::Leaf(block) => block.get_mut(index),
            Node::Inner(_, children) => {
                let left_size = children[LEFT].len();
                if left_size >= index {
                    children[LEFT].get_mut(index)
                } else {
                    children[RIGHT].get_mut(index - left_size)
                }
            }
        }
    }
}
