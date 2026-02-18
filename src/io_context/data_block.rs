use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

pub struct ReadOnlyBlock {
    data: Rc<RefCell<Vec<u8>>>,
}

impl From<Rc<RefCell<Vec<u8>>>> for ReadOnlyBlock {
    fn from(value: Rc<RefCell<Vec<u8>>>) -> Self {
        Self {
            data: value.clone(),
        }
    }
}

impl ReadOnlyBlock {
    pub fn get(&self) -> Ref<'_, Vec<u8>> {
        self.data.borrow()
    }
}

pub struct MutableBlock {
    data: Rc<RefCell<Vec<u8>>>,
}

impl From<Rc<RefCell<Vec<u8>>>> for MutableBlock {
    fn from(value: Rc<RefCell<Vec<u8>>>) -> Self {
        Self {
            data: value.clone(),
        }
    }
}

impl MutableBlock {
    pub fn get(&self) -> RefMut<'_, Vec<u8>> {
        self.data.borrow_mut()
    }
}
