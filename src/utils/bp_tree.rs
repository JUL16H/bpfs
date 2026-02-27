use std::cell::RefCell;
use std::rc::Rc;

mod bp_tree_node;

use thiserror::Error;
use zerocopy::little_endian::U64;

use crate::block_allocator::{BlockAllocateError, BlockAllocator};
use crate::block_device::BlockDeviceError;
use crate::io_context::IOContext;
use crate::utils::bp_tree::bp_tree_node::{NodeHeader, NodeParseError, NodeView, NodeViewMut};
use crate::{block_device::BlockDevice, utils::cache::Cache};

pub struct BPTree<D, C, A>
where
    D: BlockDevice,
    C: Cache<u64, Rc<RefCell<Vec<u8>>>>,
{
    io_context: Rc<RefCell<IOContext<D, C>>>,
    root_block: Option<u64>,
    first_leaf: u64,
    allocator: Option<Rc<RefCell<A>>>,
    m: u64,
}

#[derive(Error, Debug)]
pub enum BPTreeError {
    #[error("Error has occurred when try to split node")]
    FailedToSplit,
    #[error("Disk IO Error: {0}")]
    DiskError(#[from] BlockDeviceError),
    #[error("Node parse Error: {0}")]
    NodeParseError(#[from] NodeParseError),
    #[error("Failed to allocate new block while splitting  node: {0}")]
    AllocateError(#[from] BlockAllocateError),
    #[error("Empty tree")]
    EmptyTree,
}

impl<D, C, A> BPTree<D, C, A>
where
    D: BlockDevice,
    C: Cache<u64, Rc<RefCell<Vec<u8>>>>,
    A: BlockAllocator,
{
    pub fn new(ioc: Rc<RefCell<IOContext<D, C>>>, allocator: Option<Rc<RefCell<A>>>) -> Self {
        Self {
            io_context: ioc.clone(),
            root_block: None,
            allocator,
            first_leaf: u64::MAX,
            m: (ioc.borrow_mut().get_disk_block_size() - size_of::<NodeHeader>() as u64) / 16,
        }
    }

    pub fn new_as_block_manager(
        io_context: Rc<RefCell<IOContext<D, C>>>,
        beg_block: u64,
    ) -> Result<Self, BPTreeError> {
        let m: u64;
        {
            let mut ioc = io_context.borrow_mut();
            m = (ioc.get_disk_block_size() - size_of::<NodeHeader>() as u64) / 16;
            let block = ioc.get_mut(beg_block)?;
            let mut block = block.get();
            let nodeview = NodeViewMut::get_from_bytes(&mut block, m)?;
            *nodeview.header = NodeHeader::new(true, 1);
            nodeview.keys[0] = U64::new(beg_block + 1);
            nodeview.vals[0] =
                U64::new(ioc.get_disk_capacity() / ioc.get_disk_block_size() - beg_block - 1);
        }
        Ok(Self {
            io_context,
            root_block: Some(beg_block),
            allocator: None,
            first_leaf: beg_block,
            m,
        })
    }

    pub fn insert(&mut self, key: u64, val: u64) -> Result<(), BPTreeError> {
        if let Some(root_block) = self.root_block {
            #[allow(unused_assignments)]
            let mut needs_split = false;
            {
                let mut ioc = self.io_context.borrow_mut();
                let block = ioc.get_mut(root_block)?;
                let mut block = block.get();
                let nodeview = NodeViewMut::get_from_bytes(&mut block, self.m)?;
                needs_split = nodeview.header.num_keys.get() >= self.m - 1;
            };
            if needs_split {
                let new_root = self.alloc()?;
                {
                    let mut ioc = self.io_context.borrow_mut();
                    let new_block = ioc.get_mut(new_root)?;
                    let mut new_block = new_block.get();
                    let new_root_view = NodeViewMut::get_from_bytes(&mut new_block, self.m)?;
                    *new_root_view.header = NodeHeader::new(false, 0);
                    new_root_view.vals[0] = root_block.into();
                }
                self.split_node(new_root, root_block)?;
                self.root_block = Some(new_root);
            }
            self.insert_node(self.root_block.unwrap(), key, val)
        } else {
            let new_block_idx = self.alloc()?;

            self.root_block = Some(new_block_idx);
            self.first_leaf = new_block_idx;
            let new_block = self.io_context.borrow_mut().get_mut(new_block_idx)?;
            let mut new_block_guard = new_block.get();
            let new_node = NodeViewMut::get_from_bytes(&mut new_block_guard, self.m)?;
            *new_node.header = NodeHeader::new(true, 1);
            new_node.keys[0] = key.into();
            new_node.vals[0] = val.into();
            Ok(())
        }
    }

    pub fn get(&self, key: u64) -> Result<Option<u64>, BPTreeError> {
        let mut cur_block: u64;
        if let Some(root_block) = self.root_block {
            cur_block = root_block;
        } else {
            return Ok(None);
        }

        loop {
            let mut ioc = self.io_context.borrow_mut();
            let block = ioc.get(cur_block)?;
            let block = block.get();
            let nodeview = NodeView::get_from_bytes(&block, self.m)?;

            if nodeview.header.is_leaf == 1 {
                let Ok(idx) = nodeview.keys[..nodeview.header.num_keys.get() as usize]
                    .binary_search(&U64::new(key))
                else {
                    return Ok(None);
                };
                return Ok(Some(nodeview.vals[idx].into()));
            }

            let idx = nodeview.keys[..nodeview.header.num_keys.get() as usize]
                .partition_point(|&x| x.get() <= key);
            cur_block = nodeview.vals[idx].get();
        }
    }

    fn insert_node(&mut self, block_idx: u64, key: u64, val: u64) -> Result<(), BPTreeError> {
        let (next_idx, needs_split) = {
            let mut ioc = self.io_context.borrow_mut();
            let block = ioc.get_mut(block_idx)?;
            let mut block = block.get();
            let nodeview = NodeViewMut::get_from_bytes(&mut block, self.m)?;

            if nodeview.header.is_leaf == 1 {
                let num_keys = nodeview.header.num_keys.get() as usize;
                match nodeview.keys[..num_keys].binary_search(&U64::new(key)) {
                    Ok(idx) => {
                        nodeview.vals[idx] = val.into();
                    }
                    Err(idx) => {
                        nodeview.keys.copy_within(idx..num_keys, idx + 1);
                        nodeview.vals.copy_within(idx..num_keys, idx + 1);
                        nodeview.keys[idx] = key.into();
                        nodeview.vals[idx] = val.into();
                        nodeview.header.num_keys += 1;
                    }
                }
                return Ok(());
            };

            let idx = nodeview.keys[..nodeview.header.num_keys.get() as usize]
                .partition_point(|&x| x.get() <= key);

            let next_idx = nodeview.vals[idx].get();
            let next_block = ioc.get_mut(next_idx)?;
            let mut next_block = next_block.get();
            let next_nodeview = NodeViewMut::get_from_bytes(&mut next_block, self.m)?;

            (next_idx, next_nodeview.header.num_keys.get() >= self.m - 1)
        };

        if needs_split {
            self.split_node(block_idx, next_idx)?;
            return self.insert_node(block_idx, key, val);
        }
        self.insert_node(next_idx, key, val)
    }

    fn alloc(&mut self) -> Result<u64, BPTreeError> {
        if self.allocator.is_some() {
            Ok(self.allocator.as_ref().unwrap().borrow_mut().alloc()?)
        } else {
            self.pop_first_extent_block()
        }
    }

    fn split_node(&mut self, father: u64, child: u64) -> Result<(), BPTreeError> {
        let new_node = self.alloc()?;

        let mut ioc = self.io_context.borrow_mut();
        let father_block = ioc.get_mut(father)?;
        let mut father_block = father_block.get();
        let father_nodeview = NodeViewMut::get_from_bytes(&mut father_block, self.m)?;

        let child_block = ioc.get_mut(child)?;
        let mut child_block = child_block.get();
        let child_nodeview = NodeViewMut::get_from_bytes(&mut child_block, self.m)?;

        let new_node_block = ioc.get_mut(new_node)?;
        let mut new_node_block = new_node_block.get();
        let new_nodeview = NodeViewMut::get_from_bytes(&mut new_node_block, self.m)?;

        let mid = child_nodeview.header.num_keys.get() / 2;

        if child_nodeview.header.is_leaf == 0 {
            let num_left = mid;
            let num_right = child_nodeview.header.num_keys.get() - mid - 1;

            *new_nodeview.header = NodeHeader::new(false, num_right);
            new_nodeview.keys[..num_right as usize].clone_from_slice(
                &child_nodeview.keys
                    [mid as usize + 1..child_nodeview.header.num_keys.get() as usize],
            );
            new_nodeview.vals[..num_right as usize + 1].clone_from_slice(
                &child_nodeview.vals
                    [mid as usize + 1..child_nodeview.header.num_keys.get() as usize + 1],
            );

            child_nodeview.header.num_keys = U64::new(num_left);

            let insert_idx = father_nodeview.keys[..father_nodeview.header.num_keys.get() as usize]
                .partition_point(|&x| x.get() < child_nodeview.keys[mid as usize].get());
            father_nodeview.keys.copy_within(
                insert_idx..father_nodeview.header.num_keys.get() as usize,
                insert_idx + 1,
            );
            father_nodeview.vals.copy_within(
                insert_idx + 1..father_nodeview.header.num_keys.get() as usize + 1,
                insert_idx + 2,
            );
            father_nodeview.header.num_keys += 1;
            father_nodeview.keys[insert_idx] = child_nodeview.keys[mid as usize];
            father_nodeview.vals[insert_idx + 1] = new_node.into();
        } else {
            let num_left = mid;
            let num_right = child_nodeview.header.num_keys.get() - num_left;

            *new_nodeview.header = NodeHeader::new(true, num_right);
            new_nodeview.keys[..num_right as usize]
                .copy_from_slice(&child_nodeview.keys[mid as usize..(mid + num_right) as usize]);
            new_nodeview.vals[..num_right as usize]
                .copy_from_slice(&child_nodeview.vals[mid as usize..(mid + num_right) as usize]);

            child_nodeview.header.num_keys = num_left.into();

            let insert_idx = father_nodeview.keys[..father_nodeview.header.num_keys.get() as usize]
                .partition_point(|&x| x.get() < new_nodeview.keys[0].get());
            father_nodeview.keys.copy_within(
                insert_idx..father_nodeview.header.num_keys.get() as usize,
                insert_idx + 1,
            );
            father_nodeview.keys[insert_idx as usize] = new_nodeview.keys[0];
            father_nodeview.vals.copy_within(
                insert_idx + 1..father_nodeview.header.num_keys.get() as usize + 1,
                insert_idx + 2,
            );
            father_nodeview.vals[insert_idx + 1] = new_node.into();
            father_nodeview.header.num_keys += 1;
        };
        new_nodeview.header.next = child_nodeview.header.next;
        child_nodeview.header.next = new_node.into();

        Ok(())
    }

    // TODO: 处理空盘块删除
    pub fn pop_first_extent_block(&mut self) -> Result<u64, BPTreeError> {
        let mut cur_block = self.first_leaf;
        loop {
            let mut ioc = self.io_context.borrow_mut();
            let block = ioc.get_mut(cur_block)?;
            let mut block = block.get();
            let nodeview = NodeViewMut::get_from_bytes(&mut block, self.m)?;

            for i in 0..nodeview.header.num_keys.get() as usize {
                if nodeview.vals[i].get() == 0 {
                    continue;
                }

                let new_block_id = nodeview.keys[i].get();
                nodeview.keys[i] += 1;
                nodeview.vals[i] -= 1;

                let num_keys = nodeview.header.num_keys.get() as usize;
                nodeview.keys.copy_within(i..num_keys, 0);
                nodeview.vals.copy_within(i..num_keys, 0);
                nodeview.header.num_keys -= U64::new(i as u64);

                return Ok(new_block_id);
            }
            nodeview.header.num_keys = 0.into();
            cur_block = nodeview.header.next.get();
        }
    }

    pub fn get_m(&self) -> u64 {
        self.m
    }
}

#[cfg(test)]
mod test {
    use std::u8;

    use crate::{
        block_allocator::{
            bptree_allocator::BPTreeAllocator, none_allocator::NoneAllocator,
            test_allocator::TestAllocator,
        },
        block_device::mem_disk::MemDisk,
        utils::cache::lru::LRU,
    };

    use super::*;

    pub fn pseudo_random_mapper(mut x: u64) -> u64 {
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
        x ^= x >> 31;
        x
    }

    #[test]
    fn it_works() -> Result<(), BPTreeError> {
        let disk = Rc::new(RefCell::new(MemDisk::new(4 * 1024 * 1024)));
        let iocontext = Rc::new(RefCell::new(IOContext::<
            MemDisk,
            LRU<u64, Rc<RefCell<Vec<u8>>>>,
        >::new(1024, disk.clone())));
        let allocator = Rc::new(RefCell::new(BPTreeAllocator::<
            MemDisk,
            LRU<u64, Rc<RefCell<Vec<u8>>>>,
            NoneAllocator,
        >::try_new(iocontext.clone(), 0)?));

        let mut bptree = BPTree::new(iocontext.clone(), Some(allocator.clone()));
        let m = bptree.get_m();

        for i in 0..64 * m {
            let key = pseudo_random_mapper(i);
            let val = pseudo_random_mapper(key);
            bptree.insert(key, val)?;
        }

        for i in 0..64 * m {
            let key = pseudo_random_mapper(i);
            let val = pseudo_random_mapper(key);
            assert_eq!(bptree.get(key)?, Some(val));
        }

        Ok(())
    }
}
