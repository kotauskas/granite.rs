/// Types which have corresponding immutably borrowing iterators.
///
/// This is a lot like [`IntoIterator`], but analogous to [`Vec`]'s `.iter()` method.
pub trait IntoRefIterator<'a> {
    /// The item type over references to which iteration will be performed.
    type Item: 'a;
    /// The resulting iterator type.
    type Iter: Iterator<Item = &'a Self::Item>;
    /// Borrows the value into a by-reference iterator with immutable access.
    fn iter(&'a self) -> Self::Iter;
}
/// Types which have corresponding *mutably* borrowing iterators.
///
/// This is a lot like [`IntoIterator`], but analogous to [`Vec`]'s `.iter_mut()` method.
pub trait IntoMutIterator<'a> {
    /// The item type over mutable references to which iteration will be performed.
    type Item: 'a;
    /// The resulting iterator type.
    type IterMut: Iterator<Item = &'a mut Self::Item>;
    /// Borrows the value into a by-reference iterator with mutable access.
    fn iter_mut(&'a mut self) -> Self::IterMut;
}
