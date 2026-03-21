use std::{cell::RefCell, rc::Rc};

use bpfs::{
    block_allocator::{bptree_allocator::BPTreeAllocator, none_allocator::NoneAllocator},
    block_device::{file_disk::FileDisk, mem_disk::MemDisk},
    io_context::IOContext,
    utils::{
        bp_tree::{BPTree, BPTreeError},
        cache::lru::LRU,
    },
};

fn pseudo_random_mapper(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^= x >> 31;
    x
}

fn main() -> Result<(), BPTreeError> {
    let disk = Rc::new(RefCell::new(FileDisk::new(
        "disk.img",
        4 * 1024 * 1024 * 1024 * 1024,
    )));
    let iocontext = Rc::new(RefCell::new(IOContext::<
        FileDisk,
        LRU<u64, Rc<RefCell<Vec<u8>>>>,
    >::new(1024, disk.clone())));
    let allocator = Rc::new(RefCell::new(BPTreeAllocator::<
        FileDisk,
        LRU<u64, Rc<RefCell<Vec<u8>>>>,
        NoneAllocator,
    >::try_new(iocontext.clone(), 0)?));

    let mut bptree = BPTree::new(iocontext.clone(), allocator.clone());
    let m = bptree.get_m();

    let test_size = 6553500;

    // for i in 0..test_size * m {
    //     let key = pseudo_random_mapper(i);
    //     let val = pseudo_random_mapper(key);
    //     bptree.insert(key, val)?;
    // }
    //
    // for i in 0..test_size * m {
    //     let key = pseudo_random_mapper(i);
    //     let val = pseudo_random_mapper(key);
    //     assert_eq!(bptree.get(key)?, Some(val));
    // }

    for i in 0..test_size * m {
        let key = 3;
        let val = pseudo_random_mapper(i);
        bptree.insert(key, val)?;
        assert_eq!(bptree.get(key)?, Some(val));
    }

    Ok(())
}
