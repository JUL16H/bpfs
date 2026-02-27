use thiserror::Error;
use zerocopy::byteorder::little_endian::*;
use zerocopy::{CastError, FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[repr(C)]
#[derive(FromBytes, IntoBytes, Unaligned, Immutable, KnownLayout, Debug)]
pub struct NodeHeader {
    pub is_leaf: u8,
    _padding: [u8; 7],
    pub num_keys: U64,
    pub next: U64, // 0 means nullptr
}

#[derive(Error, Debug)]
pub enum NodeParseError {
    #[error("Header alignment error")]
    HeaderAligenment,
    #[error("Header size error")]
    HeaderSize,
    #[error("Kvs alignment error")]
    KvsAligenment,
    #[error("Kvs size error")]
    KvsSize,
}

impl NodeHeader {
    pub fn new(is_leaf: bool, num_keys: u64) -> Self {
        Self {
            is_leaf: if is_leaf { 1 } else { 0 },
            _padding: [0; 7],
            num_keys: num_keys.into(),
            next: U64::MAX_VALUE,
        }
    }
}

pub struct NodeViewMut<'a> {
    pub header: &'a mut NodeHeader,
    pub keys: &'a mut [U64],
    pub vals: &'a mut [U64],
}

impl<'a> NodeViewMut<'a> {
    pub fn get_from_bytes(bytes: &'a mut [u8], m: u64) -> Result<Self, NodeParseError> {
        let (header, kvs) = NodeHeader::mut_from_prefix(bytes).map_err(|e| match e {
            CastError::Alignment(_) => NodeParseError::HeaderAligenment,
            CastError::Size(_) => NodeParseError::HeaderSize,
        })?;
        let kvs = <[U64]>::mut_from_bytes(kvs).map_err(|e| match e {
            CastError::Alignment(_) => NodeParseError::KvsAligenment,
            CastError::Size(_) => NodeParseError::HeaderSize,
        })?;
        if kvs.len() < 2 * m as usize {
            return Err(NodeParseError::KvsSize);
        }
        let (keys, vals) = kvs.split_at_mut(m as usize);
        Ok(NodeViewMut { header, keys, vals })
    }
}

pub struct NodeView<'a> {
    pub header: &'a NodeHeader,
    pub keys: &'a [U64],
    pub vals: &'a [U64],
}

impl<'a> NodeView<'a> {
    pub fn get_from_bytes(bytes: &'a [u8], m: u64) -> Result<Self, NodeParseError> {
        let (header, kvs) = NodeHeader::ref_from_prefix(bytes).map_err(|e| match e {
            CastError::Alignment(_) => NodeParseError::HeaderAligenment,
            CastError::Size(_) => NodeParseError::HeaderSize,
        })?;
        let kvs = <[U64]>::ref_from_bytes(kvs).map_err(|e| match e {
            CastError::Alignment(_) => NodeParseError::KvsAligenment,
            CastError::Size(_) => NodeParseError::HeaderSize,
        })?;
        if kvs.len() < 2 * m as usize {
            return Err(NodeParseError::KvsSize);
        }
        let (keys, vals) = kvs.split_at(m as usize);
        Ok(NodeView { header, keys, vals })
    }
}
