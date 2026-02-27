use std::cell::RefCell;
use std::rc::Rc;

use crate::block_device::{BlockDevice, BlockDeviceError};
use crate::utils::cache::Cache;

pub mod data_block;
pub use data_block::{MutableBlock, ReadOnlyBlock};

pub struct IOContext<D, C> {
    cache: C,
    disk: Rc<RefCell<D>>,
    block_size: u64,
}

impl<D, C> IOContext<D, C>
where
    D: BlockDevice,
    C: Cache<u64, Rc<RefCell<Vec<u8>>>>,
{
    pub fn new(cache_size: u64, disk: Rc<RefCell<D>>) -> Self {
        Self {
            cache: C::new(cache_size),
            disk: disk.clone(),
            block_size: disk.borrow_mut().get_block_size(),
        }
    }

    pub fn get(&mut self, sector_idx: u64) -> Result<ReadOnlyBlock, BlockDeviceError> {
        if let Some(block) = self.cache.get(&sector_idx, false) {
            return Ok(block.clone().into());
        }

        let mut v = vec![0u8; self.block_size as usize];
        self.disk.borrow_mut().read(sector_idx, &mut v)?;
        let v = Rc::new(RefCell::new(v.clone()));
        if let Some(entry) = self.cache.put(sector_idx, v.clone(), false) {
            self.flush_block(entry)?;
        }

        Ok(v.into())
    }

    pub fn get_mut(&mut self, sector_idx: u64) -> Result<MutableBlock, BlockDeviceError> {
        if let Some(block) = self.cache.get(&sector_idx, true) {
            return Ok(block.clone().into());
        }

        let mut v = vec![0u8; self.block_size as usize];
        self.disk.borrow_mut().read(sector_idx, &mut v)?;
        let v = Rc::new(RefCell::new(v.clone()));
        if let Some(entry) = self.cache.put(sector_idx, v.clone(), true) {
            self.flush_block(entry)?;
        }

        Ok(v.into())
    }

    pub fn get_disk_block_size(&self) -> u64 {
        self.disk.borrow_mut().get_block_size()
    }

    pub fn get_disk_capacity(&self) -> u64 {
        self.disk.borrow_mut().get_capacity()
    }

    fn flush_block(
        &self,
        entry: (u64, Rc<RefCell<Vec<u8>>>, bool),
    ) -> Result<(), BlockDeviceError> {
        if !entry.2 {
            return Ok(());
        }
        self.disk
            .borrow_mut()
            .write(entry.0, &entry.1.borrow_mut())?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), BlockDeviceError> {
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
