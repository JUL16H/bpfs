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
}

impl FileDisk {
    pub fn new(path: &str, cap: u64) -> Self {
        assert_eq!(cap % Self::BLOCK_SIZE, 0);

        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);
        // options.custom_flags(libc::O_DIRECT);
        let file = options.open(path).expect("Failed to open file.");
        file.set_len(cap).expect("Failed to set length.");

        Self {
            path: path.to_string(),
            file,
            cap,
        }
    }

    pub fn remove(self) -> Result<(), io::Error> {
        fs::remove_file(&self.path)?;
        Ok(())
    }
}

impl BlockDevice for FileDisk {
    type Error = io::Error;

    const BLOCK_SIZE: u64 = 4096;

    fn read(&self, sector_idx: u64, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let offset = sector_idx * Self::BLOCK_SIZE;
        self.file.read_at(buffer, offset)?;
        Ok(())
    }

    fn write(&mut self, sector_idx: u64, data: &[u8]) -> Result<(), Self::Error> {
        let offset = sector_idx * Self::BLOCK_SIZE;
        self.file.write_at(data, offset)?;
        Ok(())
    }

    fn get_capacity(&self) -> u64 {
        self.cap
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() -> Result<(), io::Error> {
        let mut disk = FileDisk::new("./vdisk.img", 1024 * 1024 * 1024);

        let mut data = vec![0u8; FileDisk::BLOCK_SIZE as usize];
        let mut buffer = vec![0u8; FileDisk::BLOCK_SIZE as usize];

        for i in 0..100 {
            data[i] = i as u8;
        }

        disk.write(5, &data)?;
        disk.read(5, &mut buffer)?;

        assert_eq!(data[0..100], buffer[0..100]);

        disk.remove()?;

        Ok(())
    }
}
