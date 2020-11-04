# Granite
[![Crates.io](https://img.shields.io/crates/v/granite)](https://crates.io/crates/granite "Granite on Crates.io")
[![Docs.rs](https://img.shields.io/badge/documentation-docs.rs-informational)](https://docs.rs/granite "Granite on Docs.rs")
[![Build Status](https://github.com/kotauskas/granite.rs/workflows/Build/badge.svg)](https://github.com/kotauskas/granite.rs/actions "GitHub Actions page for Granite")
[![Minimal Supported Rust Version](https://img.shields.io/badge/msrv-1.46-orange)](https://blog.rust-lang.org/2020/08/27/Rust-1.46.0.html "Rust 1.46 release notes")

Generic backing storage framework for building arena-allocated data structures.

## Overview
Unlike many other languages, Rust is not very friendly to naive implementations of data structures based on smart pointers. The most you can represent with [`Rc`]/[`Arc`] is a [directed acyclic graph (DAG)][DAG], and the second word in that somewhat cryptic abbreviation hints why: you'll be running into cycles between the pointers, which will prevent the whole structure from ever being dropped, leaking both the memory for the nodes and their user-defined contents. The only way you can solve this is by using garbage collection, or by dropping every single element manually, while keeping track of the collection's not-yet-dropped nodes, which would require an elaborate algorithm and runtime overhead to match.

Also, performing separate memory allocations for every single element is horribly slow and unfriendly to cache locality, not unlike the universally despised [`LinkedList`]. So much for the zero-cost abstraction that Rust strives to achieve.

Enter arena-allocated data structures: a way to have data structures backed by `Vec`, which provides excellent cache locality and performs bulk allocations.

Unlike smart pointer data structures, arena allocated data structures do not store any pointers. Instead, they store keys. A key is an identifier of an element within the backing storage, unique in the scope of one instance of the backing storage. Keys may overlap between multiple storages and between an element which existed at some point but has been removed, but they may not overlap among elements coexisting in one point of time in one collection.

## Feature flags
- `alloc` (**enabled by default**) — enables support for [`Vec`] and [`VecDeque`] from the standard library, while keeping the crate `no_std`. Requires a functional global allocator.
- `arrayvec` (**enabled by default**) — enables support for [`ArrayVec`].
- `smallvec` (**enabled by default**) — enables support for [`SmallVec`].
- `slab` (**enabled by default**) — enables support for [`Slab`].
- `slotmap` — enables support for [`SlotMap`]. [`Slab`] will likely be faster because it's not versioned, so this is disabled by default.

[`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html " "
[`VecDeque`]: https://doc.rust-lang.org/std/collections/struct.VecDeque.html " "
[`SmallVec`]: https://docs.rs/smallvec/*/smallvec/struct.SmallVec.html " "
[`ArrayVec`]: https://docs.rs/arrayvec/*/arrayvec/struct.ArrayVec.html " "
[`LinkedList`]: https://doc.rust-lang.org/std/collections/struct.LinkedList.html " "
[`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html " "
[`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html " "
[DAG]: https://en.wikipedia.org/wiki/Directed_acyclic_graph " "
