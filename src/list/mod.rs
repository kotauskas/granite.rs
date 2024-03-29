#[cfg(feature = "alloc")]
mod alloc_impl;
#[cfg(feature = "arrayvec")]
mod arrayvec_impl;
#[cfg(feature = "smallvec")]
mod smallvec_impl;
#[cfg(feature = "tinyvec")]
mod tinyvec_impl;

mod sparse;
pub use sparse::{SparseStorage, Slot as SparseStorageSlot};
pub mod chain;
pub use chain::Chain;
#[cfg(feature = "alloc")]
pub use sparse::{Vec as SparseVec, VecDeque as SparseVecDeque};

use core::{
    num::{NonZeroUsize, NonZeroIsize},
    cmp::Ordering,
    hint,
    convert::TryFrom,
};
use crate::{IntoMutIterator, IntoRefIterator};

use super::Storage;

const U_ONE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };

/// Trait for list-like containers which can be the backing storage for data structures.
///
/// # Safety
/// There's a number of invariants which have to be followed by the container:
/// - The length of the storage cannot be modified in the container when it's borrowed immutably or not borrowed at all;
/// - `new` and `with_capacity` ***must*** return empty storages, i.e. those which have `len() == 0` and `is_empty() == true`;
/// - it should be impossible for the length of the storage to overflow `isize`;
/// - Calling [`get_unchecked`] or [`get_unchecked_mut`] with `self.len() > index` should *not* cause undefined behavior (otherwise, it may or may not — that is implementation specific);
/// - `insert_and_fix`/`remove_and_fix` call unsafe methods from [`MoveFix`], meaning that `insert` and `remove` must be implemented according to the contract of the methods of that trait;
/// - If an element is added at a position, it must be retrieveable in the exact same state as it was inserted until it is removed or modified using a method which explicitly does so.
/// - If [`CAPACITY`] is `Some(...)`, the [`capacity`] method is **required** to return its value.
///
/// Data structures may rely on those invariants for safety.
///
/// [`get_unchecked`]: #method.get_unchecked " "
/// [`get_unchecked_mut`]: #method.get_unchecked_mut " "
/// [`capacity`]: #method.capacity " "
/// [`CAPACITY`]: #associatedconstant.CAPACITY " "
pub unsafe trait ListStorage: Sized {
    /// The type of values in the container.
    type Element;

    /// The fixed capacity, for statically allocated storages. Storages like `SmallVec` should set this to `None`, since going above this limit is generally assumed to panic.
    const CAPACITY: Option<usize> = None;

    /// Creates an empty collection with the specified capacity.
    ///
    /// # Panics
    /// Collections with a fixed capacity should panic if the specified capacity is bigger than their actual one. Collections which use the heap are allowed to panic if an allocation cannot be performed, though using the [OOM abort mechanism] is also allowed.
    ///
    /// [OOM abort mechanism]: https://doc.rust-lang.org/std/alloc/fn.handle_alloc_error.html " "
    fn with_capacity(capacity: usize) -> Self;
    /// Inserts an element at position `index` within the collection, shifting all elements after it to the right.
    ///
    /// # Panics
    /// Required to panic if `index > len()`.
    fn insert(&mut self, index: usize, element: Self::Element);
    /// Removes and returns the element at position `index` within the vector, shifting all elements after it to the left.
    ///
    /// # Panics
    /// Required to panic if the specified index does not exist.
    fn remove(&mut self, index: usize) -> Self::Element;
    /// Returns the number of elements in the collection, also referred to as its 'length'.
    fn len(&self) -> usize;
    /// Returns a reference to the specified element in the collection, without doing bounds checking.
    ///
    /// # Safety
    /// If the specified index is out of bounds, a dangling reference will be created, causing *immediate undefined behavior*.
    unsafe fn get_unchecked(&self, index: usize) -> &Self::Element;
    /// Returns a *mutable* reference to the specified element in the collection, without doing bounds checking.
    ///
    /// # Safety
    /// If the specified index is out of bounds, a dangling reference will be created, causing *immediate undefined behavior*.
    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Self::Element;

    /// Returns a reference to the specified element in the collection, or `None` if the index is out of bounds.
    fn get(&self, index: usize) -> Option<&Self::Element> {
        if self.len() > index {
            Some(unsafe {
                // SAFETY: we just did a bounds check
                self.get_unchecked(index)
            })
        } else {
            None
        }
    }
    /// Returns a *mutable* reference to the specified element in the collection, or `None` if the index is out of bounds.
    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Element> {
        if self.len() > index {
            Some(unsafe {
                // SAFETY: we just did a bounds check
                self.get_unchecked_mut(index)
            })
        } else {
            None
        }
    }
    /// Creates a new empty collection. Dynamically-allocated collections created this way do not allocate memory.
    ///
    /// The default implementation calls `Self::with_capacity(0)`, which usually doesn't allocate for heap-based storages.
    fn new() -> Self {
        Self::with_capacity(0)
    }
    /// Returns `true` if the collection contains no elements, `false` otherwise.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Appends an element to the back of the collection.
    fn push(&mut self, element: Self::Element) {
        self.insert(self.len(), element)
    }
    /// Removes the last element from the collection and returns it, or `None` if it is empty.
    fn pop(&mut self) -> Option<Self::Element> {
        #[allow(clippy::if_not_else)] // Makes more sense this way
        if !self.is_empty() {
            Some(self.remove(self.len() - 1))
        } else {
            None
        }
    }
    /// Returns the amount of elements the collection can hold without requiring a memory allocation.
    ///
    /// The default implementation uses the current length. Implementors are heavily encouraged to override the default behavior.
    // TODO make mandatory to implement in the next major version bump
    fn capacity(&self) -> usize {
        self.len()
    }
    /// Reserves capacity for at least additional more elements to be inserted in the given collection. The collection may reserve more space to avoid frequent reallocations. After calling `reserve`, `capacity` will be greater than or equal to `self.len()` + `additional`. Does nothing if capacity is already sufficient.
    ///
    /// For collections which have a fixed capacity, this should first check for the specified amount of elements to reserve for and if it's not zero, either reallocate the collection anew or, if that is not supported, panic. The default implementation does exactly that.
    fn reserve(&mut self, additional: usize) {
        if self.len() + additional > self.capacity() {
            unimplemented!("this storage type does not support reallocation")
        }
    }
    /// Shrinks the capacity of the collection as much as possible.
    ///
    /// It will drop down as close as possible to the current length, though dynamically allocated collections may not always reallocate exactly as much as it is needed to store all elements and none more.
    ///
    /// The default implementation does nothing.
    fn shrink_to_fit(&mut self) {}
    /// Shortens the collection, keeping the first `len` elements and dropping the rest.
    ///
    /// If `len` is greater than the collection's current length, this has no effect.
    ///
    /// Note that this method has no effect on the allocated capacity of the collection.
    fn truncate(&mut self, len: usize) {
        let current_length = self.len();
        if len > current_length || current_length == 0 {
            return;
        }
        for i in (current_length - 1)..=len {
            self.remove(i);
        }
    }
    /// Inserts an element at position `index` within the collection. The items after the inserted item should be notified using the [`MoveFix`] trait or not have their indices changed at all (index changes are not guaranteed and this behavior is implementation-dependent).
    ///
    /// # Panics
    /// Same as `insert`.
    ///
    /// [`MoveFix`]: trait.MoveFix.html " "
    fn insert_and_shiftfix(&mut self, index: usize, element: Self::Element)
    where
        Self::Element: MoveFix,
    {
        self.insert(index, element);
        unsafe {
            // SAFETY: here we assume that removal actually did its job
            // (since the trait is unsafe, we can make this assumption)
            Self::Element::fix_right_shift(self, index, U_ONE);
        }
    }
    /// Removes and returns the element at position `index` within the collection. The items after the inserted item should be notified using the [`MoveFix`] trait or not change indicies at all (index changes are not guaranteed and this behavior is implementation-dependent).
    ///
    /// # Panics
    /// Same as `remove`.
    ///
    /// [`MoveFix`]: trait.MoveFix.html " "
    fn remove_and_shiftfix(&mut self, index: usize) -> Self::Element
    where
        Self::Element: MoveFix,
    {
        let element = self.remove(index);
        unsafe {
            // SAFETY: as above
            Self::Element::fix_left_shift(self, index, U_ONE);
        }
        element
    }
    /// Adds an element to the collection at an arbitrary index, returning that index. Will never shift elements around. The default implementation will call `push` and return the index of the element pushed.
    ///
    /// This method is used instead of `push` by data structures. It is overriden by `SparseStorage` with the use of a free-list for placing new elements in place of old holes.
    fn add(&mut self, element: Self::Element) -> usize {
        self.push(element);
        self.len() - 1
    }
}
unsafe impl<T, E> Storage for T
where
    T: ListStorage<Element = E>,
    E: MoveFix,
{
    type Key = usize;
    type Element = E;
    const CAPACITY: Option<usize> = <Self as ListStorage>::CAPACITY;

    fn add(&mut self, element: Self::Element) -> usize {
        <Self as ListStorage>::add(self, element)
    }
    fn remove(&mut self, index: &usize) -> Self::Element {
        <Self as ListStorage>::remove_and_shiftfix(self, *index)
    }
    fn len(&self) -> usize {
        <Self as ListStorage>::len(self)
    }
    fn with_capacity(capacity: usize) -> Self {
        <Self as ListStorage>::with_capacity(capacity)
    }
    unsafe fn get_unchecked(&self, index: &usize) -> &Self::Element {
        <Self as ListStorage>::get_unchecked(self, *index)
    }
    unsafe fn get_unchecked_mut(&mut self, index: &usize) -> &mut Self::Element {
        <Self as ListStorage>::get_unchecked_mut(self, *index)
    }
    fn contains_key(&self, index: &usize) -> bool {
        <Self as ListStorage>::len(self) > *index
    }
    fn get(&self, index: &usize) -> Option<&Self::Element> {
        <Self as ListStorage>::get(self, *index)
    }
    fn get_mut(&mut self, index: &usize) -> Option<&mut Self::Element> {
        <Self as ListStorage>::get_mut(self, *index)
    }
    fn new() -> Self {
        Self::with_capacity(0)
    }
    fn capacity(&self) -> usize {
        <Self as ListStorage>::capacity(self)
    }
    fn reserve(&mut self, additional: usize) {
        <Self as ListStorage>::reserve(self, additional)
    }
    fn shrink_to_fit(&mut self) {
        <Self as ListStorage>::shrink_to_fit(self)
    }
}

/// Trait alias for list-like containers which support indexing, addition and removal of elements and iteration.
///
/// This is automatically implemented for any type implementing [`ListStorage`] and HRTB-bounded [`IntoRefIterator`] and [`IntoMutIterator`].
pub trait List:
    ListStorage
    + for<'a> IntoRefIterator<'a, Item = <Self as ListStorage>::Element>
    + for<'a> IntoMutIterator<'a, Item = <Self as ListStorage>::Element>
{
}
impl<T> List for T where
    T: ListStorage
        + for<'a> IntoRefIterator<'a, Item = <Self as ListStorage>::Element>
        + for<'a> IntoMutIterator<'a, Item = <Self as ListStorage>::Element>
{
}

/// Trait for data structure element types to be able to correct indices towards other elements when they are moved around in the collection.
///
/// See the documentation on the individual methods for more details on the semantics of those hooks.
pub trait MoveFix: Sized {
    /// The hook to be called when the items in the collection get shifted due to an insertion or removal. `shifted_from` specifies the index from which the shift starts (first affected element), i.e. the index at which a new item was inserted or from which an item was removed. Positive values for `shifted_by` indicate a shift to the right, negative are to the left.
    ///
    /// This method is *never* called directly by storages. `fix_left_shift` and `fix_right_shift` are called instead. See those for more on how and when this method gets called.
    ///
    /// # Safety
    /// This method can ***only*** be called by `fix_left_shift` and `fix_right_shift`. All safety implications of those methods apply.
    unsafe fn fix_shift<S>(storage: &mut S, shifted_from: usize, shifted_by: NonZeroIsize)
    where
        S: ListStorage<Element = Self>;
    /// The hook to be called when an element in a collection gets moved from one location to another. `previous_index` represents its previous index before moving and is *not* guaranteed to be a valid index, `current_index` is its new index and is guaranteed to point towards a valid element.
    ///
    /// # Safety
    /// The implementor of this method may cause undefined behavior if the method was called erroneously and elements were not actually swapped.
    ///
    /// [`SparseStorage`]: struct.SparseStorage.html " "
    unsafe fn fix_move<S>(storage: &mut S, previous_index: usize, current_index: usize)
    where
        S: ListStorage<Element = Self>;
    /// The hook to be called when the items in the collection get shifted to the *left* due to a *removal*. `shifted_from` specifies the index from which the shift starts (first affected element), i.e. the index from which an item was removed.
    ///
    /// # Safety
    /// The implementor of this method may cause undefined behavior if the method was called erroneously and elements were not actually shifted.
    ///
    /// # Panics
    /// Required to panic on integer overflow when converting the `shifted_by` into a `NonZeroIsize`.
    unsafe fn fix_left_shift<S>(storage: &mut S, shifted_from: usize, shifted_by: NonZeroUsize)
    where
        S: ListStorage<Element = Self>,
    {
        Self::fix_shift(
            storage,
            shifted_from,
            NonZeroIsize::new(
                // SAFETY: there cannot be more than isize::MAX elements
                isize::try_from(shifted_by.get())
                    .unwrap_or_else(|_| hint::unreachable_unchecked())
                    .wrapping_neg(),
            )
            .expect("unexpected integer overflow"),
        );
    }
    /// The hook to be called when the items in the collection get shifted to the *right* due to an *insertion*. `shifted_from` specifies the index from which the shift starts (first affected element), i.e. the index at which a new item was inserted.
    ///
    /// # Safety
    /// The implementor of this method may cause undefined behavior if the method was called erroneously and elements were not actually shifted.
    unsafe fn fix_right_shift<S>(storage: &mut S, shifted_from: usize, shifted_by: NonZeroUsize)
    where
        S: ListStorage<Element = Self>,
    {
        Self::fix_shift(
            storage,
            shifted_from,
            NonZeroIsize::new(
                // SAFETY: there cannot be more than isize::MAX elements
                isize::try_from(shifted_by.get()).unwrap_or_else(|_| hint::unreachable_unchecked()),
            )
            .expect("unexpected integer overflow"),
        );
    }
}

/// Wrapper around a type which implements `MoveFix` by doing nothing when notified.
///
/// Annotated with `repr(transparent)`, so an `as` cast to the contained type will extract the value.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct DummyMoveFix<T: ?Sized>(pub T);
impl<T> DummyMoveFix<T> {
    /// Extracts the contained value.
    #[allow(clippy::missing_const_for_fn)] // *sigh* destructors
    pub fn into_inner(self) -> T {
        self.0
    }
}
/// Dummy implementation, does nothing when notified.
impl<T> MoveFix for DummyMoveFix<T> {
    unsafe fn fix_shift<S>(_: &mut S, _: usize, _: NonZeroIsize)
    where
        S: ListStorage<Element = Self>,
    {
    }
    unsafe fn fix_move<S>(_: &mut S, _: usize, _: usize)
    where
        S: ListStorage<Element = Self>,
    {
    }
}
impl<T> From<T> for DummyMoveFix<T> {
    fn from(op: T) -> Self {
        DummyMoveFix(op)
    }
}
impl<T: PartialEq + ?Sized> PartialEq<T> for DummyMoveFix<T> {
    fn eq(&self, other: &T) -> bool {
        &self.0 == other
    }
}
impl<T: PartialOrd + ?Sized> PartialOrd<T> for DummyMoveFix<T> {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}
