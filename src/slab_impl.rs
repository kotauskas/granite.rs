use slab::Slab;
use super::Storage;

unsafe impl<T> Storage for Slab<T> {
    type Key = usize;
    type Element = T;

    #[inline(always)]
    fn add(&mut self, element: Self::Element) -> Self::Key {
        self.insert(element)
    }
    #[inline(always)]
    fn remove(&mut self, key: &Self::Key) -> Self::Element {
        self.remove(*key)
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }
    #[inline(always)]
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }
    #[inline(always)]
    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Element {
        self.get_unchecked(*key)
    }
    #[inline(always)]
    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Element {
        self.get_unchecked_mut(*key)
    }
    #[inline(always)]
    fn contains_key(&self, key: &Self::Key) -> bool {
        self.contains(*key)
    }
    #[inline(always)]
    fn get(&self, key: &Self::Key) -> Option<&Self::Element> {
        self.get(*key)
    }
    #[inline(always)]
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Element> {
        self.get_mut(*key)
    }
    #[inline(always)]
    fn new() -> Self {
        Self::new()
    }
    #[inline(always)]
    fn capacity(&self) -> usize {
        self.capacity()
    }
    #[inline(always)]
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional)
    }
    #[inline(always)]
    fn shrink_to_fit(&mut self) {
        self.shrink_to_fit()
    }
}