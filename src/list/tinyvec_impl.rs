use core::slice;

use tinyvec::{Array, ArrayVec, SliceVec, TinyVec};
use crate::{ListStorage, IntoRefIterator, IntoMutIterator};

unsafe impl<A: Array> ListStorage for TinyVec<A> {
    type Element = A::Item;
    const CAPACITY: Option<usize> = None;

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
impl<'a, A: Array> IntoRefIterator<'a> for TinyVec<A>
where
    A::Item: 'a,
{
    type Item = A::Item;
    type Iter = slice::Iter<'a, A::Item>;
    fn iter(&'a self) -> Self::Iter {
        self.as_slice().iter()
    }
}
impl<'a, A: Array> IntoMutIterator<'a> for TinyVec<A>
where
    A::Item: 'a,
{
    type Item = A::Item;
    type IterMut = slice::IterMut<'a, A::Item>;
    fn iter_mut(&'a mut self) -> Self::IterMut {
        self.as_mut_slice().iter_mut()
    }
}

unsafe impl<A: Array> ListStorage for ArrayVec<A> {
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
        self.as_slice().len()
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
    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }
}
impl<'a, A: Array> IntoRefIterator<'a> for ArrayVec<A>
where
    A::Item: 'a,
{
    type Item = A::Item;
    type Iter = slice::Iter<'a, A::Item>;
    fn iter(&'a self) -> Self::Iter {
        self.as_slice().iter()
    }
}
impl<'a, A: Array> IntoMutIterator<'a> for ArrayVec<A>
where
    A::Item: 'a,
{
    type Item = A::Item;
    type IterMut = slice::IterMut<'a, A::Item>;
    fn iter_mut(&'a mut self) -> Self::IterMut {
        self.as_mut_slice().iter_mut()
    }
}

unsafe impl<'s, T: Default> ListStorage for SliceVec<'s, T> {
    type Element = T;
    const CAPACITY: Option<usize> = None;

    #[track_caller]
    fn with_capacity(_: usize) -> Self {
        Self::new()
    }
    fn insert(&mut self, index: usize, element: Self::Element) {
        self.insert(index, element)
    }
    fn remove(&mut self, index: usize) -> Self::Element {
        self.remove(index)
    }
    fn len(&self) -> usize {
        self.as_slice().len()
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
    #[track_caller]
    fn new() -> Self {
        unimplemented!("cannot create a SliceVec out of thin air")
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
    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }
}
impl<'s: 'a, 'a, T> IntoRefIterator<'a> for SliceVec<'s, T> {
    type Item = T;
    type Iter = slice::Iter<'a, T>;
    fn iter(&'a self) -> Self::Iter {
        self.as_slice().iter()
    }
}
impl<'s: 'a, 'a, T> IntoMutIterator<'a> for SliceVec<'s, T> {
    type Item = T;
    type IterMut = slice::IterMut<'a, T>;
    fn iter_mut(&'a mut self) -> Self::IterMut {
        self.as_mut_slice().iter_mut()
    }
}
