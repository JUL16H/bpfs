use crate::{block_device::BlockDevice, io_context::IOContext, utils::cache::lru::LRU};
use std::{cell::RefCell, rc::Rc};

pub mod block_device;
pub mod io_context;
pub mod utils;
pub mod block_allocator;

pub type BlockCache = LRU<u64, Rc<RefCell<Vec<u8>>>>;

pub struct FS<D>
where
    D: BlockDevice,
{
    io_context: Rc<RefCell<IOContext<D, BlockCache>>>,
}

impl<D: BlockDevice> FS<D> {
    pub fn new(disk: Rc<RefCell<D>>) -> Self {
        Self {
            io_context: Rc::new(RefCell::new(IOContext::new(1024, disk))),
        };
        todo!();
    }
}
