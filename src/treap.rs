use super::{Node, Priority, BLOCK_SIZE};
use itertools::Itertools;
use rand::random;
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
    /// Checks that the data structure respects its constraints.
    pub(super) fn is_valid(&self) -> bool {
        self.root.is_valid(None)
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
    pub fn insert(&mut self, index: usize, element: C) {
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
    pub fn push(&mut self, element: C) {
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
                    Node::Inner(_, _, [left, right]) => {
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
        // spread all elements directly into their final blocks
        let (mut tree, leaves) = iter.into_iter().chunks(BLOCK_SIZE / 2).into_iter().fold(
            (Vec::new(), 0),
            |(mut tree, leaves), chunk| {
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
                        // let's have a fake priority, we'll set it later
                        let merged = Node::Inner(0, size, [left_node, right_node]);
                        tree.push(Box::new(merged));
                    } else {
                        break;
                    }
                }
                (tree, leaves + 1)
            },
        );
        let right_node = tree.pop();
        if let Some(mut right_node) = right_node {
            // build the treap
            while let Some(left_node) = tree.pop() {
                let size = left_node.len() + right_node.len();
                right_node = Box::new(Node::Inner(0, size, [left_node, right_node]));
            }
            let mut treap = ITreap { root: *right_node };
            // now, fix priorities
            let mut priorities: Vec<Priority> =
                std::iter::repeat_with(random).take(leaves - 1).collect();
            priorities.sort_unstable();
            for_each_node_breadth_first(&mut treap.root, |node| match node {
                Node::Inner(priority, _, _) => *priority = priorities.pop().unwrap(),
                _ => (),
            });

            debug_assert!(treap.is_valid());
            treap
        } else {
            Default::default()
        }
    }
}

fn for_each_node_breadth_first<C, F: FnMut(&mut Node<C>)>(root: &mut Node<C>, mut op: F) {
    let mut remaining: std::collections::VecDeque<_> = std::iter::once(root).collect();
    while let Some(node) = remaining.pop_front() {
        op(node);
        match node {
            Node::Inner(_, _, children) => remaining.extend(children.iter_mut().map(|b| &mut **b)),
            _ => (),
        }
    }
}

fn intersect_ranges(r1: &Range<usize>, r2: &Range<usize>) -> Range<usize> {
    r1.start.max(r2.start)..r1.end.min(r2.end)
}
