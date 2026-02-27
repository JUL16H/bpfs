use crate::block_device::BlockDeviceError;

use super::BlockDevice;

#[derive(Debug)]
pub struct MemDisk {
    cap: usize,
    sector_cnt: usize,
    data: Vec<u8>,
}

const BLOCK_SIZE: u64 = 4 * 1024;

impl MemDisk {
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0);
        assert_eq!(cap % BLOCK_SIZE as usize, 0);
        Self {
            cap,
            sector_cnt: (cap / BLOCK_SIZE as usize),
            data: vec![0u8; cap],
        }
    }

    fn get_range_from_idx(sector_idx: usize) -> (usize, usize) {
        let start = sector_idx * BLOCK_SIZE as usize;
        let end = start + BLOCK_SIZE as usize;
        (start, end)
    }
}

impl BlockDevice for MemDisk {
    fn read(&self, sector_idx: u64, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if sector_idx as usize >= self.sector_cnt {
            return Err(BlockDeviceError::IdxOutOfRange {
                idx: sector_idx,
                max: self.sector_cnt as u64,
            });
        }
        if buffer.len() != self.get_block_size() as usize {
            return Err(BlockDeviceError::MismachedBufferSize {
                size: buffer.len() as u64,
            });
        }
        let (start, end) = Self::get_range_from_idx(sector_idx as usize);
        buffer.clone_from_slice(&self.data[start..end]);
        Ok(())
    }

    fn write(&mut self, sector_idx: u64, data: &[u8]) -> Result<(), BlockDeviceError> {
        if sector_idx >= self.sector_cnt as u64 {
            return Err(BlockDeviceError::IdxOutOfRange {
                idx: sector_idx,
                max: self.sector_cnt as u64,
            });
        }
        if data.len() != BLOCK_SIZE as usize {
            return Err(BlockDeviceError::MismachedBufferSize {
                size: data.len() as u64,
            });
        }
        let (start, end) = Self::get_range_from_idx(sector_idx as usize);
        self.data[start..end].clone_from_slice(&data);
        Ok(())
    }

    fn get_capacity(&self) -> u64 {
        self.cap as u64
    }

    fn get_block_size(&self) -> u64 {
        BLOCK_SIZE
    }
}
