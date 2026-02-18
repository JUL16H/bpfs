use std::cell::RefCell;
use std::rc::Rc;

use crate::block_device::BlockDevice;
use crate::utils::cache::Cache;

pub mod data_block;
pub use data_block::{MutableBlock, ReadOnlyBlock};

pub struct IoContext<D, C>
where
    D: BlockDevice,
    C: Cache<u64, Rc<RefCell<Vec<u8>>>>,
{
    cache: C,
    disk: Rc<RefCell<D>>,
}

impl<D, C> IoContext<D, C>
where
    D: BlockDevice,
    D::Error: std::error::Error + 'static,
    C: Cache<u64, Rc<RefCell<Vec<u8>>>>,
{
    pub fn new(cap: u64, disk: Rc<RefCell<D>>) -> Self {
        Self {
            cache: C::new(cap),
            disk,
        }
    }

    pub fn get(&mut self, sector_idx: u64) -> Result<ReadOnlyBlock, D::Error> {
        if self.cache.peek(sector_idx).is_none() {
            let mut v = vec![0u8; D::BLOCK_SIZE as usize];
            self.disk.borrow_mut().read(sector_idx, &mut v)?;
            let v = Rc::new(RefCell::new(v));
            if let Some(entry) = self.cache.put(sector_idx, v) {
                self.flush_block(entry)?;
            }
        }
        Ok(self
            .cache
            .get(&sector_idx)
            .expect("WTF. That's impossible.")
            .into())
    }

    pub fn get_mut(&mut self, sector_idx: u64) -> Result<MutableBlock, D::Error> {
        if self.cache.peek(sector_idx).is_none() {
            let mut v = vec![0u8; D::BLOCK_SIZE as usize];
            self.disk.borrow_mut().read(sector_idx, &mut v)?;
            let v = Rc::new(RefCell::new(v));
            if let Some(entry) = self.cache.put(sector_idx, v) {
                self.flush_block(entry)?;
            }
        }
        self.cache.mark_dirty(&sector_idx);
        Ok(self
            .cache
            .get(&sector_idx)
            .expect("WTF. That's impossible.")
            .into())
    }

    fn flush_block(&self, entry: (u64, Rc<RefCell<Vec<u8>>>, bool)) -> Result<(), D::Error> {
        if !entry.2 {
            return Ok(());
        }
        self.disk
            .borrow_mut()
            .write(entry.0, &entry.1.borrow_mut())?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), D::Error> {
        let disk = &mut self.disk;
        let entries = &mut self.cache.drain();
        for entry in entries {
            if !entry.2 {
                return Ok(());
            }
            disk.borrow_mut().write(entry.0, &entry.1.borrow_mut())?;
        }
        Ok(())
    }
}
