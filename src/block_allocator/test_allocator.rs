use crate::block_allocator::BlockAllocator;

pub struct TestAllocator {
    cur: u64,
}

impl TestAllocator {
    pub fn new() -> Self {
        Self { cur: 1 }
    }
}

impl BlockAllocator for TestAllocator {
    fn alloc(&mut self) -> Result<u64, super::BlockAllocateError> {
        let rst = Ok(self.cur);
        self.cur += 1;
        rst
    }

    fn free(&mut self, _idx: u64) -> Result<(), super::BlockAllocateError> {
        Ok(())
    }
}
