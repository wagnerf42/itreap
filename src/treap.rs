use super::{Node, BLOCK_SIZE};
use itertools::Itertools;
use std::ops::Range;

pub struct ITreap<C> {
    root: Node<C>,
}

impl<C> std::ops::Index<usize> for ITreap<C> {
    type Output = C;
    /// Borrows the `i`th element.
    /// Cost is O(log(n/B)).
    fn index(&self, i: usize) -> &Self::Output {
        self.root.get(i).unwrap()
    }
}

impl<C> std::ops::IndexMut<usize> for ITreap<C> {
    /// Mutably borrows the `i`th element.
    /// Cost is O(log(n/B)).
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        self.root.get_mut(i).unwrap()
    }
}

impl<C> ITreap<C> {
    /// Create a new empty indexed treap.
    pub fn new() -> Self {
        ITreap {
            root: Node::Leaf(Vec::new()),
        }
    }
    /// Inserts an element at position `index`.
    /// Cost is O(log(n/B)+B).
    ///
    /// # Example
    ///
    /// ```
    /// use itreap::ITreap;
    ///
    /// let mut t = ITreap::new();
    ///
    /// t.insert(0, 7); // [7]
    /// t.insert(0, 2); // [2,7]
    /// t.insert(1, 3); // [2,3,7]
    ///
    /// assert!(t.iter().eq(&[2, 3, 7]))
    /// ```
    pub fn insert(&mut self, index: usize, element: C) -> &mut C {
        self.root.insert(index, element)
    }
    /// Adds an element to the back.
    /// Cost is O(log(n/B)+1).
    ///
    /// # Example
    ///
    /// ```
    /// use itreap::ITreap;
    ///
    /// let mut t = ITreap::new();
    ///
    /// t.push(2);
    /// t.push(4);
    /// t.push(6);
    ///
    /// assert!(t.iter().eq(&[2, 4 ,6]))
    /// ```
    pub fn push(&mut self, element: C) -> &mut C {
        self.insert(self.len(), element)
    }
    /// Returns the number of elements in the indexed treap.
    /// Cost is O(1).
    pub fn len(&self) -> usize {
        self.root.len()
    }
    /// Loops on all elements.
    /// Cost is O(n).
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a C> + 'a {
        self.between(0..self.root.len())
    }
    /// Loop on all elements corresponding to indices in given range.
    /// Cost is O(log(n/B) + k) where k designates the number of elements we should loop upon.
    ///
    /// # Example
    ///
    /// ```
    /// use itreap::ITreap;
    ///
    /// let t:ITreap<_> = (0..10).map(|e| e*2).collect();
    /// assert!(t.between(1..4).eq(&[2, 4, 6]))
    /// ```
    pub fn between<'a>(
        &'a self,
        selection: std::ops::Range<usize>,
    ) -> impl Iterator<Item = &'a C> + 'a {
        let mut remaining_nodes = std::iter::once((&self.root, 0..self.root.len()))
            .filter(|(_, r)| !intersect_ranges(r, &selection).is_empty())
            .collect::<Vec<_>>();
        let mut current_block = None;
        let mut current_block_iter = None;
        std::iter::from_fn(move || loop {
            while current_block.is_none() && !remaining_nodes.is_empty() {
                let (next_node, next_node_range) = remaining_nodes.pop().unwrap();
                match next_node {
                    Node::Inner(_, [left, right]) => {
                        let right_start = next_node_range.start + left.len();
                        let right_range = right_start..next_node_range.end;
                        let left_range = next_node_range.start..right_start;
                        if !intersect_ranges(&right_range, &selection).is_empty() {
                            remaining_nodes.push((right, right_range));
                        }
                        if !intersect_ranges(&left_range, &selection).is_empty() {
                            remaining_nodes.push((left, left_range));
                        }
                    }
                    Node::Leaf(block) => {
                        current_block = Some(block);
                        let selected = intersect_ranges(&next_node_range, &selection);
                        let retained_elements = (selected.start - next_node_range.start)
                            ..(selected.end - next_node_range.start);
                        current_block_iter =
                            current_block.as_ref().map(|b| b[retained_elements].iter())
                    }
                }
            }
            if let Some(iter) = &mut current_block_iter {
                let maybe_next_value = iter.next();
                if let Some(next_value) = maybe_next_value {
                    return Some(next_value);
                } else {
                    current_block_iter = None;
                    current_block = None;
                }
            } else {
                return None;
            }
        })
    }
}

impl<C> std::default::Default for ITreap<C> {
    fn default() -> Self {
        ITreap::new()
    }
}

impl<C> std::iter::FromIterator<C> for ITreap<C> {
    /// Transform an iterator into an indexed treap.
    /// This will always create a perfectly balanced tree.
    /// Cost is O(n).
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        // avoid inserting elements one by one.
        // loop on all blocks
        let mut tree = iter.into_iter().chunks(BLOCK_SIZE / 2).into_iter().fold(
            Vec::new(),
            |mut tree, chunk| {
                // we keep a stack of nodes
                // and merge the last two nodes when the get equal size
                let block = chunk.collect::<Vec<_>>();
                tree.push(Box::new(Node::Leaf(block)));
                loop {
                    let l = tree.len();
                    if l >= 2 && tree[l - 1].len() == tree[l - 2].len() {
                        let right_node = tree.pop().unwrap();
                        let left_node = tree.pop().unwrap();
                        let size = left_node.len() + right_node.len();
                        let merged = Node::Inner(size, [left_node, right_node]);
                        tree.push(Box::new(merged));
                    } else {
                        break;
                    }
                }
                tree
            },
        );
        let right_node = tree.pop();
        if let Some(mut right_node) = right_node {
            while let Some(left_node) = tree.pop() {
                let size = left_node.len() + right_node.len();
                right_node = Box::new(Node::Inner(size, [left_node, right_node]));
            }
            ITreap { root: *right_node }
        } else {
            Default::default()
        }
    }
}

fn intersect_ranges(r1: &Range<usize>, r2: &Range<usize>) -> Range<usize> {
    r1.start.max(r2.start)..r1.end.min(r2.end)
}
