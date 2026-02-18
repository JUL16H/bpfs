use super::BlockDevice;

#[derive(Debug)]
pub struct MemDisk {
    cap: usize,
    sector_cnt: usize,
    data: Vec<u8>,
}

impl MemDisk {
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0);
        assert_eq!(cap % Self::BLOCK_SIZE as usize, 0);
        Self {
            cap,
            sector_cnt: (cap / Self::BLOCK_SIZE as usize),
            data: vec![0u8; cap],
        }
    }

    fn get_range_from_idx(sector_idx: usize) -> (usize, usize) {
        let start = sector_idx * Self::BLOCK_SIZE as usize;
        let end = start + Self::BLOCK_SIZE as usize;
        (start, end)
    }
}

#[derive(Debug)]
pub enum MemDiskIOError {
    IdxOutOfRange { idx: usize, max: usize },
    MismachedBufferSize { size: usize },
}

impl BlockDevice for MemDisk {
    type Error = MemDiskIOError;
    const BLOCK_SIZE: u64 = 1024;

    fn read(&self, sector_idx: u64, buffer: &mut [u8]) -> Result<(), Self::Error> {
        if sector_idx as usize >= self.sector_cnt {
            return Err(MemDiskIOError::IdxOutOfRange {
                idx: sector_idx as usize,
                max: self.sector_cnt,
            });
        }
        if buffer.len() != Self::BLOCK_SIZE as usize {
            return Err(MemDiskIOError::MismachedBufferSize { size: buffer.len() });
        }
        let (start, end) = Self::get_range_from_idx(sector_idx as usize);
        buffer.clone_from_slice(&self.data[start..end]);
        Ok(())
    }

    fn write(&mut self, sector_idx: u64, data: &[u8]) -> Result<(), Self::Error> {
        if sector_idx >= self.sector_cnt as u64 {
            return Err(MemDiskIOError::IdxOutOfRange {
                idx: sector_idx as usize,
                max: self.sector_cnt,
            });
        }
        if data.len() != Self::BLOCK_SIZE as usize {
            return Err(MemDiskIOError::MismachedBufferSize { size: data.len() });
        }
        let (start, end) = Self::get_range_from_idx(sector_idx as usize);
        self.data[start..end].clone_from_slice(&data);
        Ok(())
    }

    fn get_capacity(&self) -> u64 {
        self.cap as u64
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() -> Result<(), MemDiskIOError> {
        let mut mem = MemDisk::new(10 * MemDisk::BLOCK_SIZE as usize);

        let mut data = vec![0u8; MemDisk::BLOCK_SIZE as usize];
        let mut buffer = vec![0u8; MemDisk::BLOCK_SIZE as usize];

        for i in 0..100 {
            data[i] = i as u8;
        }

        mem.write(3, &data)?;
        mem.read(3, &mut buffer)?;

        assert_eq!(data[0..100], buffer[0..100]);

        Ok(())
    }
}
