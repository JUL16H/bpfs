use thiserror::Error;

pub mod file_disk;
pub mod mem_disk;

pub trait BlockDevice {
    fn read(&self, sector_idx: u64, buffer: &mut [u8]) -> Result<(), BlockDeviceError>;
    fn write(&mut self, sector_idx: u64, data: &[u8]) -> Result<(), BlockDeviceError>;
    fn get_capacity(&self) -> u64;
    fn get_block_size(&self) -> u64;
}

#[derive(Error, Debug)]
pub enum BlockDeviceError {
    #[error("Index out of range: idx {idx}, max {max}")]
    IdxOutOfRange { idx: u64, max: u64 },
    #[error("Mismached buffer size: {size}")]
    MismachedBufferSize { size: u64 },
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
}
