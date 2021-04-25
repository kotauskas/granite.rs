use arrayvec::{ArrayVec, Array};
use super::ListStorage;

unsafe impl<A> ListStorage for ArrayVec<A>
where
    A: Array,
{
    type Element = A::Item;
    const CAPACITY: Option<usize> = Some(A::CAPACITY);

    fn with_capacity(capacity: usize) -> Self {
        assert!(
            capacity <= A::CAPACITY,
            "requested capacity is bigger than the underlying array size"
        );
        Self::new()
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
        self.as_slice().get_unchecked(index)
    }
    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Self::Element {
        self.as_mut_slice().get_unchecked_mut(index)
    }

    fn get(&self, index: usize) -> Option<&Self::Element> {
        self.as_slice().get(index)
    }
    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Element> {
        self.as_mut_slice().get_mut(index)
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
        A::CAPACITY
    }
    fn reserve(&mut self, additional: usize) {
        if self.len() + additional > self.capacity() {
            unimplemented!("ArrayVec does not support allocating memory; if you need such functionality, use SmallVec instead")
        }
    }
    fn shrink_to_fit(&mut self) {}
    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }
}
