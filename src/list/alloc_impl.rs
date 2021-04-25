use core::{hint, slice};
use alloc::{
    vec::Vec,
    collections::vec_deque::{self, VecDeque},
};
use crate::{IntoMutIterator, IntoRefIterator, ListStorage};

unsafe impl<T> ListStorage for Vec<T> {
    type Element = T;

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
impl<'a, T: 'a> IntoRefIterator<'a> for Vec<T> {
    type Item = T;
    type Iter = slice::Iter<'a, T>;
    fn iter(&'a self) -> Self::Iter {
        (&self[..]).iter()
    }
}
impl<'a, T: 'a> IntoMutIterator<'a> for Vec<T> {
    type Item = T;
    type IterMut = slice::IterMut<'a, T>;
    fn iter_mut(&'a mut self) -> Self::IterMut {
        (&mut self[..]).iter_mut()
    }
}

unsafe impl<T> ListStorage for VecDeque<T> {
    type Element = T;

    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }
    fn insert(&mut self, index: usize, element: Self::Element) {
        self.insert(index, element)
    }
    fn remove(&mut self, index: usize) -> Self::Element {
        self.remove(index).expect("index out of bounds")
    }
    fn len(&self) -> usize {
        self.len()
    }
    unsafe fn get_unchecked(&self, index: usize) -> &Self::Element {
        // FIXME this relies on LLVM being smart enough to optimize out the bounds check
        self.get(index)
            .unwrap_or_else(|| hint::unreachable_unchecked())
    }
    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Self::Element {
        // FIXME see above
        self.get_mut(index)
            .unwrap_or_else(|| hint::unreachable_unchecked())
    }

    fn get(&self, index: usize) -> Option<&Self::Element> {
        self.get(index)
    }
    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Element> {
        self.get_mut(index)
    }
    fn push(&mut self, element: Self::Element) {
        self.push_back(element)
    }
    fn pop(&mut self) -> Option<Self::Element> {
        self.pop_back()
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
impl<'a, T: 'a> IntoRefIterator<'a> for VecDeque<T> {
    type Item = T;
    type Iter = vec_deque::Iter<'a, T>;
    fn iter(&'a self) -> Self::Iter {
        self.iter()
    }
}
impl<'a, T: 'a> IntoMutIterator<'a> for VecDeque<T> {
    type Item = T;
    type IterMut = vec_deque::IterMut<'a, T>;
    fn iter_mut(&'a mut self) -> Self::IterMut {
        self.iter_mut()
    }
}

/*
// TODO reimplement LinkedList with a custom SmartLinkedList
#[cfg(feature = "linked_list_storage")]
use alloc::collections::LinkedList;

#[cfg(feature = "linked_list_storage")]
unsafe impl<T> ListStorage for LinkedList<T> {
    type Element = T;

        fn with_capacity(capacity: usize) -> Self {
        assert_eq!(capacity, 0, "cannot create a linked list with nonzero preallocated capacity");
        Self::new()
    }
        fn insert(&mut self, index: usize, element: Self::Element) {
        assert!(index <= self.len(), "incorrect index");
        let mut cursor = {
            let mut cursor = self.cursor_front_mut();
            for _ in 0..index {
                cursor.move_next();
            }
            cursor
        };
        cursor.insert_after(element)
    }
        fn remove(&mut self, index: usize) -> Self::Element {
        let mut cursor = {
            let mut cursor = self.cursor_front_mut();
            for _ in 0..index {
                cursor.move_next();
            }
            cursor
        };
        cursor.remove_current().expect("index out of bounds")
    }
        fn len(&self) -> usize {
        self.len()
    }
        unsafe fn get_unchecked(&self, index: usize) -> &Self::Element {
        self.get(index).unwrap_or_else(|| unreachable_unchecked())
    }
        unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Self::Element {
        self.get_mut(index).unwrap_or_else(|| unreachable_unchecked())
    }
        fn get(&self, index: usize) -> Option<&Self::Element> {
        self.iter().nth(index)
    }
        fn get_mut(&mut self, index: usize) -> Option<&mut Self::Element> {
        self.iter_mut().nth(index)
    }

        fn new() -> Self {
        Self::new()
    }
        fn push(&mut self, element: Self::Element) {
        self.push_back(element)
    }
        fn pop(&mut self) -> Option<Self::Element> {
        self.pop_back()
    }
        fn reserve(&mut self, additional: usize) {
        if self.len() + additional > self.capacity() {
            unimplemented!("linked lists are always at max capacity")
        }
    }
}
*/
