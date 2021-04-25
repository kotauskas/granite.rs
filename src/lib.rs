//! Generic backing storage framework for building arena-allocated data structures.
//!
//! # Overview
//! Unlike many other languages, Rust is not very friendly to naive implementations of data structures based on smart pointers. The most you can represent with [`Rc`]/[`Arc`] is a [directed acyclic graph (DAG)][DAG], and the second word in that somewhat cryptic abbreviation hints why: you'll be running into cycles between the pointers, which will prevent the whole structure from ever being dropped, leaking both the memory for the nodes and their user-defined contents. The only way you can solve this is by using garbage collection, or by dropping every single element manually, while keeping track of the collection's not-yet-dropped nodes, which would require an elaborate algorithm and runtime overhead to match.
//!
//! Also, performing separate memory allocations for every single element is horribly slow and unfriendly to cache locality, not unlike the universally despised [`LinkedList`]. So much for the zero-cost abstraction that Rust strives to achieve.
//!
//! Enter arena-allocated data structures: a way to have data structures backed by `Vec`, which provides excellent cache locality and performs bulk allocations.
//!
//! Unlike smart pointer data structures, arena allocated data structures do not store any pointers. Instead, they store keys. A key is an identifier of an element within the backing storage, unique in the scope of one instance of the backing storage. Keys may overlap between multiple storages and between an element which existed at some point but has been removed, but they may not overlap among elements coexisting in one point of time in one collection.
//!
//! # Public dependencies
//! - `arrayvec` — `^0.5`
//! - `smallvec` — `^1.4`
//! - `slab` — `^0.4`
//! - `slotmap` — `^0.4`
//!
//! PRs are welcome from those interested in those version numbers being modified.
//!
//! # Feature flags
//! - `alloc` (**enabled by default**) — enables support for [`Vec`] and [`VecDeque`] from the standard library, while keeping the crate `no_std`. Requires a functional global allocator, though only at runtime and not at compile time.
//! - `arrayvec` — enables support for [`ArrayVec`].
//! - `smallvec` — enables support for [`SmallVec`].
//! - `slab` — enables support for [`Slab`].
//! - `slotmap` — enables support for [`SlotMap`], [`HopSlotMap`] and [`DenseSlotMap`]. [`Slab`] will likely be faster because it's not versioned, so this is disabled by default.
//! - `union_optimizations` — forwarded to Granite, adds some layout optimizations by using untagged unions, decreasing memory usage in `SparseStorage`. **Requires a nightly compiler** (see [tracking issue for RFC 2514]) and thus is disabled by default.
//!
//! [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html " "
//! [`VecDeque`]: https://doc.rust-lang.org/std/collections/struct.VecDeque.html " "
//! [`SmallVec`]: https://docs.rs/smallvec/*/smallvec/struct.SmallVec.html " "
//! [`ArrayVec`]: https://docs.rs/arrayvec/*/arrayvec/struct.ArrayVec.html " "
//! [`SlotMap`]: https://docs.rs/slotmap/*/slotmap/struct.SlotMap.html " "
//! [`HopSlotMap`]: https://docs.rs/slotmap/*/slotmap/hop/struct.HopSlotMap.html " "
//! [`DenseSlotMap`]: https://docs.rs/slotmap/*/slotmap/dense/struct.DenseSlotMap.html " "
//! [`Slab`]: https://docs.rs/slab/*/slab/struct.Slab.html " "
//! [`LinkedList`]: https://doc.rust-lang.org/std/collections/struct.LinkedList.html " "
//! [`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html " "
//! [`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html " "
//! [DAG]: https://en.wikipedia.org/wiki/Directed_acyclic_graph " "
//! [tracking issue for RFC 2514]: https://github.com/rust-lang/rust/issues/55149 " "

#![warn(
    rust_2018_idioms,
    clippy::cargo,
    clippy::nursery,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    // Broken, will display warnings even for undocumented items, including trait impls
    //missing_doc_code_examples,
    unused_qualifications,
    variant_size_differences,
    clippy::pedantic,
    clippy::unwrap_used, // Only .expect() allowed
)]
#![deny(anonymous_parameters, bare_trait_objects)]
#![allow(
    clippy::use_self,
    clippy::must_use_candidate,
    clippy::module_name_repetitions
)]
#![no_std]
#![cfg_attr(feature = "doc_cfg", feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod list;
pub use list::*;

mod iter;
pub use iter::*;

#[cfg(feature = "slab")]
mod slab_impl;
#[cfg(feature = "slotmap")]
mod slotmap_impl;

use core::fmt::Debug;

/// Trait for various kinds of containers which can be the backing storage for data structures.
///
/// # Safety
/// There's a number of invariants which have to be followed by the container:
/// - The length of the storage cannot be modified in the container when it's borrowed immutably or not borrowed at all;
/// - `new` and `with_capacity` ***must*** return empty storages, i.e. those which have `len() == 0` and `is_empty() == true`;
/// - it should be impossible for the length of the storage to overflow `usize`;
/// - Calling [`get_unchecked`] or [`get_unchecked_mut`] if `contains_key` on the same key returns `true` should *not* cause undefined behavior (otherwise, it may or may not — that is implementation specific);
/// - Calling `remove` if `contains_key` on the same key should *never* panic or only perform an aborting panic (i.e. not allowing unwinding), as that might leave the data structure in an invalid state during some operations;
/// - If an element is added at a key, it must be retrieveable in the exact same state as it was inserted until it is removed or modified using a method which explicitly does so.
/// - If [`CAPACITY`] is `Some(...)`, the [`capacity`] method is **required** to return its value.
///
/// Data structures may rely on those invariants for safety.
///
/// [`get_unchecked`]: #method.get_unchecked " "
/// [`get_unchecked_mut`]: #method.get_unchecked_mut " "
/// [`capacity`]: #method.capacity " "
/// [`CAPACITY`]: #associatedconstant.CAPACITY " "
pub unsafe trait Storage: Sized {
    /// The type used for element naming.
    type Key: Clone + Debug + Eq;
    /// The type of the elements stored.
    type Element;

    /// The fixed capacity, for statically allocated storages. Storages like `SmallVec` should set this to `None`, since going above this limit is generally assumed to panic.
    const CAPACITY: Option<usize> = None;

    /// Adds an element to the collection with an unspecified key, returning that key.
    fn add(&mut self, element: Self::Element) -> Self::Key;
    /// Removes and returns the element identified by `key` within the storage.
    ///
    /// # Panics
    /// Required to panic if the specified key does not exist.
    fn remove(&mut self, key: &Self::Key) -> Self::Element;
    /// Returns the number of elements in the storage, also referred to as its 'length'.
    fn len(&self) -> usize;
    /// Creates an empty collection with the specified capacity.
    ///
    /// # Panics
    /// Collections with a fixed capacity should panic if the specified capacity is bigger than their actual one. Collections which use the heap are allowed to panic if an allocation cannot be performed, though using the [OOM abort mechanism] is also allowed.
    ///
    /// [OOM abort mechanism]: https://doc.rust-lang.org/std/alloc/fn.handle_alloc_error.html " "
    fn with_capacity(capacity: usize) -> Self;
    /// Returns a reference to the specified element in the storage, without checking for presence of the key inside the collection.
    ///
    /// # Safety
    /// If the element at the specified key is not present in the storage, a dangling reference will be created, causing *immediate undefined behavior*.
    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Element;
    /// Returns a *mutable* reference to the specified element in the storage, without checking for presence of the key inside the collection.
    ///
    /// # Safety
    /// If the element at the specified key is not present in the storage, a dangling reference will be created, causing *immediate undefined behavior*.
    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Element;
    /// Returns `true` if the specified key is present in the storage, `false` otherwise.
    ///
    /// If this method returned `true`, calling `get_unchecked`/`get_unchecked_mut` on the same key is guaranteed to be safe.
    fn contains_key(&self, key: &Self::Key) -> bool;

    /// Returns a reference to the specified element in the collection, or `None` if the key is not present in the storage.
    fn get(&self, key: &Self::Key) -> Option<&Self::Element> {
        if self.contains_key(key) {
            Some(unsafe {
                // SAFETY: we just checked for key presence
                self.get_unchecked(key)
            })
        } else {
            None
        }
    }
    /// Returns a *mutable* reference to the specified element in the collection, or `None` if the key is not present in the storage.
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Element> {
        if self.contains_key(key) {
            Some(unsafe {
                // SAFETY: we just checked for key presence
                self.get_unchecked_mut(key)
            })
        } else {
            None
        }
    }
    /// Creates a new empty storage. Dynamically-allocated storages created this way do not allocate memory.
    ///
    /// The default implementation calls `Self::with_capacity(0)`, which usually doesn't allocate for heap-based storages.
    fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Returns `true` if the storage contains no elements, `false` otherwise.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Returns the amount of elements the collection can hold without requiring a memory allocation.
    ///
    /// The default implementation uses the current length. Implementors are heavily encouraged to override the default behavior.
    // TODO make mandatory to implement in the next major version bump
    fn capacity(&self) -> usize {
        self.len()
    }
    /// Reserves capacity for at least additional more elements to be inserted in the given storage. The storage may reserve more space to avoid frequent reallocations. After calling `reserve`, `capacity` will be greater than or equal to `self.len()` + `additional`. Does nothing if capacity is already sufficient.
    ///
    /// For storages which have a fixed capacity, this should first check for the specified amount of elements to reserve for and if it's not zero, either reallocate the collection anew or, if that is not supported, panic. The default implementation does exactly that.
    fn reserve(&mut self, additional: usize) {
        if self.len() + additional > self.capacity() {
            unimplemented!("this storage type does not support reallocation")
        }
    }
    /// Shrinks the capacity of the storage as much as possible.
    ///
    /// It will drop down as close as possible to the current length, though dynamically allocated storages may not always reallocate exactly as much as it is needed to store all elements and none more.
    ///
    /// The default implementation does nothing.
    fn shrink_to_fit(&mut self) {}
}

/// The default storage type used by data structures when a storage type is not provided.
///
/// This is chosen according to the following strategy:
/// - If the `slab` feature flag is enabled, [`Slab`] is used
/// - Otherwise, if `alloc` is enabled, [`SparseVec`] is used
/// - Otherwise, if `smallvec` is enabled, a [*sparse*][`SparseStorage`] [`SmallVec`] *with zero-sized backing storage* is used
/// - Otherwise, if `arrayvec` is enabled, an [`ArrayVec`] *with zero-sized backing storage* is used
/// - If even `arrayvec` is not available, a compile error is generated.
/// No other storage types are ever used as defaults.
///
/// [`Slab`]: https://docs.rs/slab/*/slab/struct.Slab.html " "
/// [`SparseVec`]: type.SparseVec.html " "
/// [`SmallVec`]: https://docs.rs/smallvec/*/smallvec/struct.SmallVec.html " "
/// [`ArrayVec`]: https://docs.rs/arrayvec/*/arrayvec/struct.ArrayVec.html " "
/// [`SparseStorage`]: struct.SparseStorage.html " "
pub type DefaultStorage<T> = _DefaultStorage<T>;

#[cfg(feature = "alloc")]
type _DefaultStorage<T> = SparseVec<T>;

#[cfg(all(feature = "smallvec", not(feature = "alloc")))]
type _DefaultStorage<T> = SparseStorage<smallvec::SmallVec<[T; 0]>, T>;

#[cfg(all(
    feature = "arrayvec",
    not(feature = "alloc"),
    not(feature = "smallvec"),
))]
type _DefaultStorage<T> = arrayvec::ArrayVec<[T; 0]>;
#[cfg(all(
    not(feature = "arrayvec"),
    not(feature = "alloc"),
    not(feature = "smallvec"),
))]
compile_error!(
    "\
cannot pick default storage, please choose at least one type of storage to be the default"
);
