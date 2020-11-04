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
//! # Feature flags
//! - `alloc` (**enabled by default**) — enables support for [`Vec`] and [`VecDeque`] from the standard library, while keeping the crate `no_std`. Requires a functional global allocator.
//! - `arrayvec` (**enabled by default**) — enables support for [`ArrayVec`].
//! - `smallvec` (**enabled by default**) — enables support for [`SmallVec`].
//! - `slab` (**enabled by default**) — enables support for [`Slab`].
//! - `slotmap` — enables support for [`SlotMap`]. [`Slab`] will likely be faster because it's not versioned, so this is disabled by default.
//!
//! [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html " "
//! [`VecDeque`]: https://doc.rust-lang.org/std/collections/struct.VecDeque.html " "
//! [`SmallVec`]: https://docs.rs/smallvec/*/smallvec/struct.SmallVec.html " "
//! [`ArrayVec`]: https://docs.rs/arrayvec/*/arrayvec/struct.ArrayVec.html " "
//! [`LinkedList`]: https://doc.rust-lang.org/std/collections/struct.LinkedList.html " "
//! [`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html " "
//! [`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html " "
//! [DAG]: https://en.wikipedia.org/wiki/Directed_acyclic_graph " "

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
    clippy::cast_lossless,
    clippy::await_holding_lock,
    clippy::checked_conversions,
    clippy::copy_iterator,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_iter_loop,
    clippy::explicit_into_iter_loop,
    clippy::filter_map,
    clippy::filter_map_next,
    clippy::find_map,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::fn_params_excessive_bools,
    clippy::implicit_hasher,
    clippy::implicit_saturating_sub,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::items_after_statements,
    clippy::large_stack_arrays,
    clippy::let_unit_value,
    clippy::macro_use_imports,
    clippy::match_same_arms,
    clippy::match_wild_err_arm,
    clippy::match_wildcard_for_single_variants,
    // sick of this stupid lint, disabling
    // clippy::module_name_repetitions,
    clippy::mut_mut,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::option_option,
    clippy::pub_enum_variant_names,
    clippy::range_plus_one,
    clippy::range_minus_one,
    clippy::redundant_closure_for_method_calls,
    clippy::same_functions_in_if_condition,
    // also sick of this one, gives too much false positives inherent to its design
    // clippy::shadow_unrelated,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::string_add_assign,
    clippy::too_many_lines,
    clippy::type_repetition_in_bounds,
    clippy::trivially_copy_pass_by_ref,
    clippy::unicode_not_nfc,
    clippy::unnested_or_patterns,
    clippy::unsafe_derive_deserialize,
    clippy::unused_self,
    clippy::used_underscore_binding,
    clippy::clone_on_ref_ptr,
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::filetype_is_file,
    clippy::get_unwrap,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::unneeded_field_pattern,
    clippy::unwrap_used, // Only .expect() allowed
    clippy::use_debug,
    clippy::verbose_file_reads,
    clippy::wrong_pub_self_convention,
)]
#![deny(
    anonymous_parameters,
    bare_trait_objects,
)]
#![allow(clippy::use_self)] // Broken

#![no_std]
#![cfg_attr(feature = "cfg_doc", feature(cfg_doc))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod list;
pub use list::*;

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
///
/// Data structures may rely on those invariants for safety.
///
/// [`get_unchecked`]: #method.get_unchecked " "
/// [`get_unchecked_mut`]: #method.get_unchecked_mut " "
pub unsafe trait Storage: Sized {
    /// The type used for element naming.
    type Key: Clone + Debug + Eq;
    /// The type of the elements stored.
    type Element;

    /// Adds an element to the collection with an unspecified key, returning that key.
    fn add(&mut self, element: Self::Element) -> Self::Key;
    /// Removes and returns the element identified by `key` within the storage.
    ///
    /// # Panics
    /// Required to panic if the specified key does not exist.
    fn remove(&mut self, key: &Self::Key) -> Self::Element;
    /// Returns the number of elements in the storage, also referred to as its 'length'.
    fn len(&self) -> usize;
    /// Creates an empty storage with the specified capacity.
    ///
    /// # Panics
    /// Storages with a fixed capacity should panic if the specified capacity does not match their actual one, and are recommended to override the `new` method to use the correct capacity.
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
    #[inline]
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
    #[inline]
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
    /// Storages with fixed capacity should override this method to use the correct capacity, as the default implementation calls `Self::with_capacity(0)`.
    #[inline(always)]
    fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Returns `true` if the storage contains no elements, `false` otherwise.
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Returns the amount of elements the storage can hold without requiring a memory allocation.
    ///
    /// For storages which have a fixed capacity, this should be equal to the length; the default implementation uses exactly that.
    #[inline(always)]
    fn capacity(&self) -> usize {
        self.len()
    }
    /// Reserves capacity for at least additional more elements to be inserted in the given storage. The storage may reserve more space to avoid frequent reallocations. After calling `reserve`, `capacity` will be greater than or equal to `self.len()` + `additional`. Does nothing if capacity is already sufficient.
    ///
    /// For storages which have a fixed capacity, this should first check for the specified amount of elements to reserve for and if it's not zero, either reallocate the collection anew or, if that is not supported, panic. The default implementation does exactly that.
    #[inline(always)]
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
    #[inline(always)]
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
compile_error!("\
cannot pick default storage, please choose at least one type of storage to be the default"
);