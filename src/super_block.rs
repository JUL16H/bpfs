use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned, little_endian::U64};

pub const MAGIC_NUMBER: u64 = 0x3e9a;

#[repr(C)]
#[derive(FromBytes, IntoBytes, Unaligned, Immutable, KnownLayout, Debug)]
pub struct SuperBlock {
    pub magic: U64,

    pub block_size: U64,
    pub blocks_count: U64,
    pub free_blocks_count: U64,

    pub free_blocks_manager_block: U64,
    pub free_inodes_manager_block: U64,
}
