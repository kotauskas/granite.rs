use smallvec::{SmallVec, Array};
use super::ListStorage;

unsafe impl<A> ListStorage for SmallVec<A>
where
    A: Array,
{
    type Element = A::Item;

    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }
    fn insert(&mut self, index: usize, element: Self::Element) {
        self.insert(index, element)
    }
    fn remove(&mut self, index: usize) -> Self::Element {
        self.remove(index)
    }
    fn len(&self) -> usize {
        self.len()
    }
    unsafe fn get_unchecked(&self, index: usize) -> &Self::Element {
        (**self).get_unchecked(index)
    }
    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Self::Element {
        (**self).get_unchecked_mut(index)
    }

    fn get(&self, index: usize) -> Option<&Self::Element> {
        (**self).get(index)
    }
    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Element> {
        (**self).get_mut(index)
    }
    fn new() -> Self {
        Self::new()
    }
    fn push(&mut self, element: Self::Element) {
        self.push(element)
    }
    fn pop(&mut self) -> Option<Self::Element> {
        self.pop()
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
    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }
}
