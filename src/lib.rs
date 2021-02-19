mod node;
pub(crate) use node::{Node, Priority, BLOCK_SIZE};

mod treap;
pub use treap::ITreap;

#[cfg(test)]
mod tests {
    use super::{ITreap, BLOCK_SIZE};
    #[test]
    fn collect() {
        let r = 0..10 * BLOCK_SIZE;
        assert!(r.clone().collect::<ITreap<_>>().iter().copied().eq(r));
    }
}
