use crate::block_device::BlockDeviceError;

use super::BlockDevice;
use std::{
    fs::{self, File, OpenOptions},
    io,
    os::unix::fs::{FileExt, OpenOptionsExt},
};

#[derive(Debug)]
pub struct FileDisk {
    path: String,
    file: File,
    cap: u64,
    block_cnt: u64,
}

const BLOCK_SIZE: u64 = 4 * 1024;

impl FileDisk {
    pub fn new(path: &str, cap: u64) -> Self {
        assert_eq!(cap % BLOCK_SIZE, 0);

        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);
        // options.custom_flags(libc::O_DIRECT);
        let file = options.open(path).expect("Failed to open file.");
        file.set_len(cap).expect("Failed to set length.");

        Self {
            path: path.to_string(),
            file,
            cap,
            block_cnt: cap / BLOCK_SIZE,
        }
    }

    pub fn remove(self) -> Result<(), io::Error> {
        fs::remove_file(&self.path)?;
        Ok(())
    }
}

impl BlockDevice for FileDisk {
    fn read(&self, block_idx: u64, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if block_idx >= self.block_cnt {
            return Err(BlockDeviceError::IdxOutOfRange {
                idx: block_idx,
                max: self.block_cnt,
            });
        }
        let offset = block_idx * self.get_block_size();
        self.file.read_at(buffer, offset)?;
        Ok(())
    }

    fn write(&mut self, sector_idx: u64, data: &[u8]) -> Result<(), BlockDeviceError> {
        let offset = sector_idx * self.get_block_size();
        self.file.write_at(data, offset)?;
        Ok(())
    }

    fn get_capacity(&self) -> u64 {
        self.cap
    }

    fn get_block_size(&self) -> u64 {
        BLOCK_SIZE
    }
}
