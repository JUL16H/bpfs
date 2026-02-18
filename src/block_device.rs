pub mod file_disk;
pub mod mem_disk;

pub trait BlockDevice {
    type Error;
    const BLOCK_SIZE: u64;
    fn read(&self, sector_idx: u64, buffer: &mut [u8]) -> Result<(), Self::Error>;
    fn write(&mut self, sector_idx: u64, data: &[u8]) -> Result<(), Self::Error>;
    fn get_capacity(&self) -> u64;
}
