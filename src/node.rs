pub(super) const BLOCK_SIZE: usize = 1000;
pub(super) const LEFT: usize = 0;
pub(super) const RIGHT: usize = 1;

pub(super) enum Node<C> {
    Leaf(Vec<C>),
    Inner(usize, [Box<Node<C>>; 2]),
}

impl<C> Node<C> {
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
        let mut block = Vec::new();
        match self {
            Node::Leaf(inner_block) => std::mem::swap(inner_block, &mut block),
            _ => unreachable!(),
        };
        let size = block.len();
        let right_block = block.split_off(size / 2);
        *self = Node::Inner(
            size,
            [
                Box::new(Node::Leaf(block)),
                Box::new(Node::Leaf(right_block)),
            ],
        );
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
