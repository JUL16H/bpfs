use super::Cache;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug)]
struct Node<K, V> {
    key: K,
    val: V,
    dirty: bool,
    prev: u64,
    next: u64,
}

pub struct LRU<K, V> {
    cap: u64,
    nodes: Vec<Node<K, V>>,
    map: HashMap<K, u64>,
    head: Option<u64>,
}

impl<K: Debug, V: Debug> Debug for LRU<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(head) = self.head {
            let mut cur = head as usize;
            for _ in 0..self.nodes.len() {
                write!(
                    f,
                    "({:?}, {:?}), ",
                    self.nodes[cur].key, self.nodes[cur].val
                )?;
                cur = self.nodes[cur].next as usize;
            }
        } else {
            write!(f, "Empty")?;
        }
        Ok(())
    }
}

impl<K, V> LRU<K, V> {
    fn detach(&mut self, idx: u64) {
        let prev = self.nodes[idx as usize].prev;
        let next = self.nodes[idx as usize].next;
        self.nodes[next as usize].prev = prev;
        self.nodes[prev as usize].next = next;
        // self.entries[idx].next = idx;
        // self.entries[idx].prev = idx;
    }

    fn attach(&mut self, idx: u64, pos: u64) {
        let next = self.nodes[pos as usize].next;
        self.nodes[idx as usize].prev = pos;
        self.nodes[idx as usize].next = next;
        self.nodes[pos as usize].next = idx;
        self.nodes[next as usize].prev = idx;
    }

    fn move_to_head(&mut self, idx: u64) {
        let head = self.head.unwrap();
        if idx == head {
            return;
        }
        self.detach(idx);
        let tail = self.nodes[self.head.unwrap() as usize].prev;
        self.attach(idx, tail);
        self.head = Some(idx);
    }
}

impl<K: Clone + Hash + Eq, V> Cache<K, V> for LRU<K, V> {
    fn new(cap: u64) -> Self {
        assert!(cap > 0);
        Self {
            cap,
            nodes: Vec::with_capacity(cap as usize),
            map: HashMap::with_capacity(cap as usize),
            head: None,
        }
    }

    fn put(&mut self, key: K, val: V) -> Option<(K, V, bool)> {
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx as usize].val = val;
            self.nodes[idx as usize].dirty = false;
            self.move_to_head(idx);
            return None;
        }

        if self.nodes.len() < self.cap as usize {
            let idx = self.nodes.len() as u64;
            self.nodes.push(Node {
                key: key.clone(),
                val,
                dirty: false,
                prev: idx,
                next: idx,
            });

            if let Some(_) = self.head {
                self.move_to_head(idx);
            } else {
                self.head = Some(idx);
            }
            self.map.insert(key, idx);
            return None;
        }

        let head = self.head.unwrap();
        let idx = self.nodes[head as usize].prev as usize;

        let old_key = std::mem::replace(&mut self.nodes[idx].key, key.clone());
        let old_val = std::mem::replace(&mut self.nodes[idx].val, val);
        let old_dirty = std::mem::replace(&mut self.nodes[idx].dirty, false);

        self.map.remove(&old_key);
        self.map.insert(key, idx as u64);

        self.move_to_head(idx as u64);
        return Some((old_key, old_val, old_dirty));
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(&idx) = self.map.get(key) {
            self.move_to_head(idx);
            self.head = Some(idx);
            Some(&mut self.nodes[idx as usize].val)
        } else {
            None
        }
    }

    fn mark_dirty(&mut self, key: &K) -> bool {
        if let Some(&idx) = self.map.get(key) {
            self.nodes[idx as usize].dirty = true;
            return true;
        }
        false
    }

    fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    fn drain(&mut self) -> impl Iterator<Item = (K, V, bool)> {
        self.map.clear();
        self.head = None;
        self.nodes
            .drain(..)
            .map(|node| (node.key, node.val, node.dirty))
    }

    fn peek(&self, key: K) -> Option<&V> {
        if let Some(&idx) = self.map.get(&key) {
            Some(&self.nodes[idx as usize].val)
        } else {
            None
        }
    }
}
