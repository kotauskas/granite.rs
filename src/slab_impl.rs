use slab::Slab;
use super::Storage;

unsafe impl<T> Storage for Slab<T> {
    type Key = usize;
    type Element = T;

    fn add(&mut self, element: Self::Element) -> Self::Key {
        self.insert(element)
    }
    fn remove(&mut self, key: &Self::Key) -> Self::Element {
        self.remove(*key)
    }
    fn len(&self) -> usize {
        self.len()
    }
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }
    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Element {
        self.get_unchecked(*key)
    }
    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Element {
        self.get_unchecked_mut(*key)
    }
    fn contains_key(&self, key: &Self::Key) -> bool {
        self.contains(*key)
    }
    fn get(&self, key: &Self::Key) -> Option<&Self::Element> {
        self.get(*key)
    }
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Element> {
        self.get_mut(*key)
    }
    fn new() -> Self {
        Self::new()
    }
    fn capacity(&self) -> usize {
        self.capacity()
    }
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional)
    }
    fn shrink_to_fit(&mut self) {
        self.shrink_to_fit()
    }
}
