use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockAllocateError {
    #[error("Index out of range")]
    IdxOutOfRange,
    #[error("There have no free disk blocks")]
    NoFreeBlocks,
}

pub trait BlockAllocator {
    fn alloc(&mut self) -> Result<u64, BlockAllocateError>;
    fn free(&mut self, idx: u64) -> Result<(), BlockAllocateError>;
}

pub mod bptree_allocator;
pub mod none_allocator;
#[cfg(test)]
pub mod test_allocator;
