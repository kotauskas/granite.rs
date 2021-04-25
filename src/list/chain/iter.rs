use core::{
    fmt::{self, Debug, Formatter},
    iter::FusedIterator,
};
use crate::{List, IntoRefIterator, IntoMutIterator};

/// An iterator over the storages of a [`Chain`].
pub struct Iter<'a, S: List + 'a, I: List<Element = S>>(<I as IntoRefIterator<'a>>::Iter);
impl<'a, S: List, I: List<Element = S>> Iterator for Iter<'a, S, I> {
    type Item = StorageProxy<'a, S>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(StorageProxy(self.0.next()?))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl<'a, S: List + 'a, I: List<Element = S>> DoubleEndedIterator for Iter<'a, S, I>
where
    <I as IntoRefIterator<'a>>::Iter: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(StorageProxy(self.0.next_back()?))
    }
}
impl<'a, S: List + 'a, I: List<Element = S>> ExactSizeIterator for Iter<'a, S, I>
where
    <I as IntoRefIterator<'a>>::Iter: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<'a, S: List + 'a, I: List<Element = S>> FusedIterator for Iter<'a, S, I> where
    <I as IntoRefIterator<'a>>::Iter: FusedIterator
{
}
impl<'a, S: List, I: List<Element = S>> Copy for Iter<'a, S, I> where
    <I as IntoRefIterator<'a>>::Iter: Copy
{
}
impl<'a, S: List, I: List<Element = S>> Clone for Iter<'a, S, I>
where
    <I as IntoRefIterator<'a>>::Iter: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<'a, S: List, I: List<Element = S>> Debug for Iter<'a, S, I>
where
    <I as IntoRefIterator<'a>>::Iter: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("chain::Iter").field(&self.0).finish()
    }
}

/// An mutable iterator over the storages of a [`Chain`].
pub struct IterMut<'a, S: List + 'a, I: List<Element = S>>(<I as IntoMutIterator<'a>>::IterMut);
impl<'a, S: List, I: List<Element = S>> Iterator for IterMut<'a, S, I> {
    type Item = StorageProxyMut<'a, S>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(StorageProxyMut(self.0.next()?))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl<'a, S: List + 'a, I: List<Element = S>> DoubleEndedIterator for IterMut<'a, S, I>
where
    <I as IntoMutIterator<'a>>::IterMut: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(StorageProxyMut(self.0.next_back()?))
    }
}
impl<'a, S: List + 'a, I: List<Element = S>> ExactSizeIterator for IterMut<'a, S, I>
where
    <I as IntoMutIterator<'a>>::IterMut: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<'a, S: List + 'a, I: List<Element = S>> FusedIterator for IterMut<'a, S, I> where
    <I as IntoMutIterator<'a>>::IterMut: FusedIterator
{
}
impl<'a, S: List, I: List<Element = S>> Debug for IterMut<'a, S, I>
where
    <I as IntoMutIterator<'a>>::IterMut: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("chain::IterMut").field(&self.0).finish()
    }
}

/// A reference to a buffer from a [`Chain`] which only allows iteration over references to elements.
pub struct StorageProxy<'a, S: List>(&'a S);
impl<'a: 'b, 'b, S: List> IntoRefIterator<'b> for StorageProxy<'a, S> {
    type Item = S::Element;
    type Iter = <S as IntoRefIterator<'b>>::Iter;
    fn iter(&'b self) -> Self::Iter {
        self.0.iter()
    }
}
impl<'a, S: List> Copy for StorageProxy<'a, S> {}
impl<'a, S: List> Clone for StorageProxy<'a, S> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, S: List> Debug for StorageProxy<'a, S>
where
    &'a S: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("StorageProxy").field(&self.0).finish()
    }
}

/// A reference to a buffer from a [`Chain`] which only allows iteration over mutable references to elements.
pub struct StorageProxyMut<'a, S: List>(&'a mut S);
impl<'a: 'b, 'b, S: List> IntoMutIterator<'b> for StorageProxyMut<'a, S> {
    type Item = S::Element;
    type IterMut = <S as IntoMutIterator<'b>>::IterMut;
    fn iter_mut(&'b mut self) -> Self::IterMut {
        self.0.iter_mut()
    }
}
impl<'a, S: List> Debug for StorageProxyMut<'a, S>
where
    &'a mut S: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("StorageProxyMut").field(&self.0).finish()
    }
}
