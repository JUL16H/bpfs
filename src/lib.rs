use std::{cell::RefCell, rc::Rc};
use thiserror::Error;
use zerocopy::{FromBytes, little_endian::U64};

use crate::{
    block_allocator::{bptree_allocator::BPTreeAllocator, none_allocator::NoneAllocator},
    block_device::{BlockDevice, BlockDeviceError},
    io_context::IOContext,
    super_block::{MAGIC_NUMBER, SuperBlock},
    utils::{bp_tree::BPTreeError, cache::Cache},
};

pub mod block_allocator;
pub mod block_device;
pub mod io_context;
pub mod super_block;
pub mod utils;

#[derive(Error, Debug)]
pub enum FsError {
    #[error("Block device error: {0}")]
    BlockDeviceError(#[from] BlockDeviceError),
    #[error("Failed to read super block from disk")]
    ReadSuperBlockError,
    #[error("B+ Tree Error: {0}")]
    BPTreeError(#[from] BPTreeError),
}

pub struct FS<D, C, A> {
    io_context: Rc<RefCell<IOContext<D, C>>>,
    block_manager: A,
    super_block: SuperBlock,
}

const CACHE_SIZE: u64 = 1024;

impl<D, C> FS<D, C, BPTreeAllocator<D, C, NoneAllocator>>
where
    D: BlockDevice,
    C: Cache<u64, Rc<RefCell<Vec<u8>>>>,
{
    pub fn try_new(disk: Rc<RefCell<D>>) -> Result<Self, FsError> {
        let io_context = Rc::new(RefCell::new(IOContext::<D, C>::new(CACHE_SIZE, disk)));
        let block_manager = BPTreeAllocator::try_new(io_context.clone(), 114514)?;

        let sb_block = io_context.borrow_mut().get(0)?;
        let sb_block = sb_block.get();
        let super_block =
            SuperBlock::read_from_bytes(&sb_block).map_err(|_| FsError::ReadSuperBlockError)?;

        let mut fs = Self {
            io_context,
            block_manager,
            super_block,
        };

        if fs.super_block.magic.get() != MAGIC_NUMBER {
            fs.formatting()?;
        };

        Ok(fs)
    }

    pub fn formatting(&mut self) -> Result<(), FsError> {
        let mut ioc = self.io_context.borrow_mut();
        ioc.clear_cache();

        let blocks_count = ioc.get_disk_capacity() / ioc.get_disk_block_size();

        self.super_block = SuperBlock {
            magic: MAGIC_NUMBER.into(),

            block_size: ioc.get_disk_block_size().into(),
            blocks_count: blocks_count.into(),
            free_blocks_count: U64::new(blocks_count) - 1,

            free_blocks_manager_block: todo!(),
            free_inodes_manager_block: todo!(),
        };
        todo!()
    }
}
