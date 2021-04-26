//! [A list data structure][Chain] wrapping a list of lists used to prevent lag spikes when dealing with huge numbers of elements.
//!
//! See the [struct-level documentation][Chain] for more.

use core::{hint, mem};
use crate::{List, ListStorage, MoveFix};

mod usize_and_flag;
use usize_and_flag::UsizeAndFlag;
mod iter;
pub use iter::*;

/// A list data structure wrapping a list of lists used to prevent lag spikes when dealing with huge numbers of elements.
///
/// The usual heap array — `Vec` — has a major weakness if it is used to store large amounts of data: horrible lag spikes will occur during insertions and when capacity is exhausted and more elements need to be inserted (reallocation and relocation of a huge data set). Copying 256 bytes is not a major performance problem, copying 1024 bytes is something that you can afford to do once in a while, and copying 1 GiB is definitely not something you'd want to do, especially in performance-critical code.
///
/// Insertion problems can be partially solved by wrapping the `Vec` in a `SparseStorage` and its `insert_and_shiftfix`, but only if `remove_and_shiftfix` is also used at least exactly as much as `insert_and_shiftfix`. Insertions without removals and full reallocations, however, are left unsolved.
///
/// `Chain` resolves the issue by limiting the capacity to which a single list storage can be used (technically doesn't have to be `Vec`, though that's what makes the most sense to use), allocating small buffers anew without touching bigger old ones if more space is requested.
#[derive(Copy, Clone, Debug)]
pub struct Chain<T, S, I>
where
    I: List<Element = S>,
    S: List<Element = T>,
{
    contents: I,
    len: usize,
    limit: UsizeAndFlag,
}
impl<T, S, I> Chain<T, S, I>
where
    S: List<Element = T>,
    I: List<Element = S>,
{
    const DEFAULT_LIMIT: usize = {
        let base = 2048 / mem::size_of::<T>();
        if base < 2 {
            // Cannot have less than 2 here, since the value has to be a
            // multiple of 2 due to UsizeAndFlag repurposing the lowest bit.
            2
        } else {
            base
        }
    };
    const DEFAULT_ALLOCATE_TO_LIMIT: bool = true;

    /// Creates an iterator over references to the storages of the chain.
    ///
    /// This allows for efficient iteration over the elements of a chain, especially if the index collection type (the list which contains the individual sub-lists) is a linked list or any other data structure with O(n) indexing. However, this requires an explicit nested loop, since creating an iterator struct which would flatten iterators into one contiguous iteration sequence is impossible, because that would require modelling a self-referential struct with lifetimes, which isn't currently possible.
    ///
    /// Also, the iterator doesn't actually iterate over references to storages: it iterates over [*proxies*](StorageProxy) instead, which are wrappers around references to storages with the sole purpose of making all other trait implementations and methods inaccessible to make sure the chain upholds the `ListStorage` safety contract in the presence of interior mutability in the used storage type.
    pub fn iter(&self) -> Iter<'_, S, I> {
        Iter(self.contents.iter())
    }
    /// Creates an iterator over mutable references to the storages of the chain.
    ///
    /// This is the same as [`iter`] — so keep all considerations which apply to it in mind — but the [proxies](StorageProxyMut) wrap mutable references and not immutable ones.
    pub fn iter_mut(&mut self) -> IterMut<'_, S, I> {
        IterMut(self.contents.iter_mut())
    }

    /// Sets the limit (**in elements, not bytes**) to which a buffer will be used before an additional allocation will be performed.
    ///
    /// The limit must be even, i.e. a multiple of 2, due to how the limit is stored internally. Check out the source code if you're curious.
    pub fn set_limit(&mut self, mut limit: usize) {
        limit &= UsizeAndFlag::SIZE_MASK;
        limit = if limit < 2 { 2 } else { limit };
        self.limit.set_size(limit)
    }
    /// Returns the currently set limit to which a buffer will be used before an additional allocation will be performed.
    pub fn limit(&self) -> usize {
        self.limit.size()
    }
    /// Sets whether additional buffers will be allocated to the limit right away or will first allocate as much as needed and only then rellocate to the limit. Enabled by default.
    pub fn allocate_to_limit(&mut self, allocate_to_limit: bool) {
        self.limit.set_flag(allocate_to_limit)
    }
    /// Returns whether [`allocate_to_limit`] is enabled.
    ///
    /// [`allocate_to_limit`]: #method.allocate_to_limit " "
    pub fn allocates_to_limit(&self) -> bool {
        self.limit.flag()
    }
    /// Returns the number of separate storages used.
    pub fn num_storages(&self) -> usize {
        self.contents.len()
    }
    fn push_allocated_storage(&mut self) -> &mut S {
        self.contents.push(S::with_capacity(self.limit()));
        unsafe {
            // SAFETY: we just pushed that; see contract on ListStorage
            self.contents
                .get_unchecked_mut(self.contents.len().wrapping_sub(1))
        }
    }
    fn push_empty_storage(&mut self) -> &mut S {
        self.contents.push(S::new());
        unsafe {
            // SAFETY: as above
            self.contents
                .get_unchecked_mut(self.contents.len().wrapping_sub(1))
        }
    }
    fn push_storage(&mut self) -> &mut S {
        if self.allocates_to_limit() {
            self.push_allocated_storage()
        } else {
            self.push_empty_storage()
        }
    }
    fn last_mut(&mut self) -> Option<&mut S> {
        self.contents
            .len()
            .checked_sub(1)
            .and_then(move |i| self.contents.get_mut(i))
    }
}

unsafe impl<T, S, I> ListStorage for Chain<T, S, I>
where
    S: List<Element = T>,
    I: List<Element = S>,
{
    type Element = T;
    const CAPACITY: Option<usize> = {
        if let (Some(index_capacity), Some(buffer_capacity)) = (I::CAPACITY, S::CAPACITY) {
            Some(index_capacity * buffer_capacity)
        } else {
            None
        }
    };
    fn with_capacity(capacity: usize) -> Self {
        let mut contents = I::new();
        contents.push(S::with_capacity(capacity));
        Self {
            contents,
            len: 0,
            limit: UsizeAndFlag::new(Self::DEFAULT_LIMIT, Self::DEFAULT_ALLOCATE_TO_LIMIT),
        }
    }
    #[track_caller]
    fn insert(&mut self, mut index: usize, element: Self::Element) {
        for st in self.contents.iter_mut() {
            let (new_index, is_end) = index.overflowing_sub(st.len());
            if is_end {
                st.insert(index, element);
                self.len += 1;
                return;
            }
            index = new_index;
        }
        panic!("index out of bounds")
    }
    #[track_caller]
    fn remove(&mut self, mut index: usize) -> Self::Element {
        for st in self.contents.iter_mut() {
            let (new_index, is_end) = index.overflowing_sub(st.len());
            if is_end {
                self.len -= 1;
                return st.remove(index);
            }
            index = new_index;
        }
        panic!("index out of bounds")
    }
    fn len(&self) -> usize {
        self.len
    }
    unsafe fn get_unchecked(&self, mut index: usize) -> &Self::Element {
        for st in self.contents.iter() {
            let (new_index, is_end) = index.overflowing_sub(st.len());
            if is_end {
                return st.get_unchecked(index);
            }
            index = new_index;
        }
        hint::unreachable_unchecked()
    }
    unsafe fn get_unchecked_mut(&mut self, mut index: usize) -> &mut Self::Element {
        for st in self.contents.iter_mut() {
            let (new_index, is_end) = index.overflowing_sub(st.len());
            if is_end {
                return st.get_unchecked_mut(index);
            }
            index = new_index;
        }
        hint::unreachable_unchecked()
    }

    fn get(&self, mut index: usize) -> Option<&Self::Element> {
        for st in self.contents.iter() {
            let (new_index, is_end) = index.overflowing_sub(st.len());
            if is_end {
                return st.get(index);
            }
            index = new_index;
        }
        None
    }
    fn get_mut(&mut self, mut index: usize) -> Option<&mut Self::Element> {
        for st in self.contents.iter_mut() {
            let (new_index, is_end) = index.overflowing_sub(st.len());
            if is_end {
                return st.get_mut(index);
            }
            index = new_index;
        }
        None
    }
    fn new() -> Self {
        Self {
            contents: I::new(),
            len: 0,
            limit: UsizeAndFlag::new(Self::DEFAULT_LIMIT, Self::DEFAULT_ALLOCATE_TO_LIMIT),
        }
    }
    fn push(&mut self, element: Self::Element) {
        let limit = self.limit();
        let storage = self
            .last_mut()
            .and_then(|last| if last.len() < limit { Some(last) } else { None });
        #[allow(clippy::option_if_let_else)] // Epic borrow checker fail
        if let Some(st) = storage {
            st.push(element);
        } else {
            self.push_storage().push(element);
        }
        self.len += 1;
    }
    fn pop(&mut self) -> Option<Self::Element> {
        #[allow(clippy::option_if_let_else)] // Same here
        if let Some(val) = self.last_mut().and_then(|storage| storage.pop()) {
            self.len -= 1;
            Some(val)
        } else {
            None
        }
    }
    fn capacity(&self) -> usize {
        let mut capacity = 0;
        for st in self.contents.iter() {
            capacity += st.capacity();
        }
        capacity
    }
    fn reserve(&mut self, additional: usize) {
        let limit = self.limit();
        let num_full_storages = additional / limit;
        let reserve_in_last_storage = additional % limit;
        for _ in 0..num_full_storages {
            self.push_allocated_storage();
        }
        self.contents
            .push(S::with_capacity(reserve_in_last_storage));
    }
    fn shrink_to_fit(&mut self) {
        let mut i = 0;
        let mut j = self.contents.len();
        while i < j {
            let st = unsafe {
                // SAFETY: see contract of ListStorage
                self.contents.get_unchecked_mut(i)
            };
            if st.len() == 0 {
                self.remove(0);
                j -= 1;
            } else {
                st.shrink_to_fit();
                i += 1;
            }
        }
    }
    fn truncate(&mut self, mut len: usize) {
        for st in self.contents.iter_mut() {
            let (new_len, is_end) = len.overflowing_sub(st.len());
            if is_end {
                st.truncate(len);
                return;
            }
            len = new_len;
        }
    }
    #[track_caller]
    fn insert_and_shiftfix(&mut self, mut index: usize, element: Self::Element)
    where
        Self::Element: MoveFix,
    {
        for st in self.contents.iter_mut() {
            let (new_index, is_end) = index.overflowing_sub(st.len());
            if is_end {
                st.insert_and_shiftfix(index, element);
                return;
            }
            index = new_index;
        }
        panic!("index out of bounds")
    }
    #[track_caller]
    fn remove_and_shiftfix(&mut self, mut index: usize) -> Self::Element
    where
        Self::Element: MoveFix,
    {
        for st in self.contents.iter_mut() {
            let (new_len, is_end) = index.overflowing_sub(st.len());
            if is_end {
                return st.remove_and_shiftfix(index);
            }
            index = new_len;
        }
        panic!("index out of bounds")
    }
    /// Uses `push` under the hood. The `Chain` should be wrapped in `SparseStorage`, not vice versa.
    fn add(&mut self, element: Self::Element) -> usize {
        self.push(element);
        self.len - 1
    }
}
