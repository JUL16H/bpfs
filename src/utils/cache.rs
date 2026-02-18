pub mod lru;

pub trait Cache<K, V> {
    fn new(cap: u64) -> Self;
    fn put(&mut self, key: K, val: V, dirty: bool) -> Option<(K, V, bool)>;
    fn get(&mut self, key: &K, dirty: bool) -> Option<&V>;
    fn mark_dirty(&mut self, key: &K) -> bool;
    fn is_empty(&self) -> bool;
    fn drain(&mut self) -> impl Iterator<Item = (K, V, bool)>;
    fn peek(&self, key: K) -> Option<&V>;
}
