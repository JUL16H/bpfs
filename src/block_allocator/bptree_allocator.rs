use std::{cell::RefCell, rc::Rc};

use super::BlockAllocator;
use crate::{
    IOContext,
    block_allocator::BlockAllocateError,
    block_device::BlockDevice,
    utils::{
        bp_tree::{BPTree, BPTreeError},
        cache::Cache,
    },
};

pub struct BPTreeAllocator<D, C, A>
where
    D: BlockDevice,
    C: Cache<u64, Rc<RefCell<Vec<u8>>>>,
    A: BlockAllocator,
{
    bptree: BPTree<D, C, A>,
}

impl<D, C, A> BPTreeAllocator<D, C, A>
where
    D: BlockDevice,
    C: Cache<u64, Rc<RefCell<Vec<u8>>>>,
    A: BlockAllocator,
{
    pub fn try_new(ioc: Rc<RefCell<IOContext<D, C>>>, beg_block: u64) -> Result<Self, BPTreeError> {
        Ok(Self {
            bptree: BPTree::new_as_block_manager(ioc, beg_block)?,
        })
    }
}

impl<D, C, A> BlockAllocator for BPTreeAllocator<D, C, A>
where
    D: BlockDevice,
    C: Cache<u64, Rc<RefCell<Vec<u8>>>>,
    A: BlockAllocator,
{
    fn alloc(&mut self) -> Result<u64, BlockAllocateError> {
        self.bptree
            .pop_first_extent_block()
            .map_err(|_| BlockAllocateError::NoFreeBlocks)
    }

    fn free(&mut self, idx: u64) -> Result<(), super::BlockAllocateError> {
        todo!()
    }
}
