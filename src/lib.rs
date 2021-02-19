mod node;
pub(crate) use node::{Node, BLOCK_SIZE};

mod treap;
pub use treap::ITreap;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
