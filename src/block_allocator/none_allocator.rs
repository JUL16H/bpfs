use crate::block_allocator::BlockAllocator;

pub struct NoneAllocator {}

impl BlockAllocator for NoneAllocator {
    fn alloc(&mut self) -> Result<u64, super::BlockAllocateError> {
        panic!()
    }
    fn free(&mut self, idx: u64) -> Result<(), super::BlockAllocateError> {
        panic!()
    }
}
