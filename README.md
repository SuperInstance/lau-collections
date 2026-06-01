# lau-collections

> Custom data structures optimized for game workloads

## What This Does

Custom data structures optimized for game workloads. Part of the PLATO/LAU ecosystem — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## The Key Idea

This crate implements the core abstractions needed for its domain, with a focus on correctness, composability, and conservation guarantees. Every public type is serializable (serde), every algorithm is tested, and every invariant is verified.

## Install

```bash
cargo add lau-collections
```

## Quick Start

See the API Reference below for complete usage. Key entry points:

```rust
use lau_collections::*;
// See types and methods below for complete usage
```

## API Reference

```rust
pub struct SparseSet 
    pub fn new() -> Self 
    pub fn insert(&mut self, index: u32, value: f64) 
    pub fn remove(&mut self, index: u32) -> bool 
    pub fn get(&self, index: u32) -> Option<f64> 
    pub fn get_mut(&mut self, index: u32) -> Option<&mut f64> 
    pub fn contains(&self, index: u32) -> bool 
    pub fn len(&self) -> usize  self.dense.len() }
    pub fn is_empty(&self) -> bool  self.dense.is_empty() }
    pub fn iter(&self) -> impl Iterator<Item = (u32, f64)> + '_ 
    pub fn clear(&mut self) 
pub struct SlotVec<T: Clone> 
    pub fn new() -> Self 
    pub fn push(&mut self, item: T) -> usize 
    pub fn remove(&mut self, index: usize) -> Option<T> 
    pub fn get(&self, index: usize) -> Option<&T> 
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> 
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> + '_ 
    pub fn len(&self) -> usize  self.len }
    pub fn is_empty(&self) -> bool  self.len == 0 }
    pub fn capacity(&self) -> usize  self.items.len() }
pub struct BitSet 
    pub fn new(size: usize) -> Self 
    pub fn set(&mut self, index: usize) 
    pub fn clear(&mut self, index: usize) 
    pub fn toggle(&mut self, index: usize) 
    pub fn is_set(&self, index: usize) -> bool 
    pub fn count(&self) -> usize 
    pub fn union(&self, other: &BitSet) -> BitSet 
    pub fn intersection(&self, other: &BitSet) -> BitSet 
    pub fn difference(&self, other: &BitSet) -> BitSet 
    pub fn is_subset(&self, other: &BitSet) -> bool 
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ 
    pub fn size(&self) -> usize  self.size }
    pub fn is_empty(&self) -> bool  self.bits.iter().all(|&w| w == 0) }
pub enum FreeSlot 
pub struct FreeList 
    pub fn new() -> Self 
    pub fn alloc(&mut self, value: u64) -> (usize, u64) 
    pub fn dealloc(&mut self, index: usize, gen: u64) -> bool 
    pub fn get(&self, index: usize, _gen: u64) -> Option<u64> 
    pub fn is_occupied(&self, index: usize) -> bool 
    pub fn occupied_count(&self) -> usize 
pub enum SmallVec<T, const N: usize> 
    pub fn new() -> Self 
    pub fn push(&mut self, item: T) 
    pub fn pop(&mut self) -> Option<T> 
    pub fn len(&self) -> usize 
    pub fn is_empty(&self) -> bool  self.len() == 0 }
    pub fn is_inline(&self) -> bool  matches!(self, Self::Inline { .. }) }
    pub fn get(&self, i: usize) -> Option<&T> 
    pub fn get_mut(&mut self, i: usize) -> Option<&mut T> 
```

## How It Works

Read the source in `src/` for full implementation details. All algorithms are documented with inline comments explaining the mathematical foundations.

## The Math

This crate implements formal mathematical constructs. See the source documentation for theorem statements and proofs of correctness.

## Testing

**49 tests** covering construction, serialization, correctness properties, edge cases, and composability with other lau-* crates.

## License

MIT
