use serde::{Deserialize, Serialize};
use std::ops::Index;

// ---------------------------------------------------------------------------
// SparseSet
// ---------------------------------------------------------------------------

/// A sparse set mapping dense `u32` indices to `f64` values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseSet {
    dense: Vec<u32>,
    sparse: Vec<Option<usize>>,
    values: Vec<f64>,
}

impl SparseSet {
    pub fn new() -> Self {
        SparseSet { dense: Vec::new(), sparse: Vec::new(), values: Vec::new() }
    }

    pub fn insert(&mut self, index: u32, value: f64) {
        let idx = index as usize;
        if idx >= self.sparse.len() {
            self.sparse.resize(idx + 1, None);
        }
        match self.sparse[idx] {
            Some(pos) => { self.values[pos] = value; }
            None => {
                let pos = self.dense.len();
                self.sparse[idx] = Some(pos);
                self.dense.push(index);
                self.values.push(value);
            }
        }
    }

    pub fn remove(&mut self, index: u32) -> bool {
        let idx = index as usize;
        if idx >= self.sparse.len() { return false; }
        match self.sparse[idx] {
            None => false,
            Some(pos) => {
                let last = self.dense.len() - 1;
                if pos != last {
                    let last_idx = self.dense[last] as usize;
                    self.dense[pos] = self.dense[last];
                    self.values[pos] = self.values[last];
                    self.sparse[last_idx] = Some(pos);
                }
                self.dense.pop();
                self.values.pop();
                self.sparse[idx] = None;
                true
            }
        }
    }

    pub fn get(&self, index: u32) -> Option<f64> {
        let idx = index as usize;
        if idx >= self.sparse.len() { return None; }
        self.sparse[idx].map(|pos| self.values[pos])
    }

    pub fn get_mut(&mut self, index: u32) -> Option<&mut f64> {
        let idx = index as usize;
        if idx >= self.sparse.len() { return None; }
        self.sparse[idx].map(move |pos| &mut self.values[pos])
    }

    pub fn contains(&self, index: u32) -> bool {
        let idx = index as usize;
        idx < self.sparse.len() && self.sparse[idx].is_some()
    }

    pub fn len(&self) -> usize { self.dense.len() }
    pub fn is_empty(&self) -> bool { self.dense.is_empty() }

    pub fn iter(&self) -> impl Iterator<Item = (u32, f64)> + '_ {
        self.dense.iter().zip(self.values.iter()).map(|(&i, &v)| (i, v))
    }

    pub fn clear(&mut self) {
        for &i in &self.dense { self.sparse[i as usize] = None; }
        self.dense.clear();
        self.values.clear();
    }
}

impl Default for SparseSet {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// SlotVec
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotVec<T: Clone> {
    items: Vec<Option<T>>,
    free: Vec<usize>,
    len: usize,
}

impl<T: Clone> SlotVec<T> {
    pub fn new() -> Self {
        SlotVec { items: Vec::new(), free: Vec::new(), len: 0 }
    }

    pub fn push(&mut self, item: T) -> usize {
        match self.free.pop() {
            Some(idx) => {
                self.items[idx] = Some(item);
                self.len += 1;
                idx
            }
            None => {
                let idx = self.items.len();
                self.items.push(Some(item));
                self.len += 1;
                idx
            }
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.items.len() { return None; }
        let old = self.items[index].take();
        if old.is_some() { self.free.push(index); self.len -= 1; }
        old
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index).and_then(|o| o.as_ref())
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index).and_then(|o| o.as_mut())
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> + '_ {
        self.items.iter().enumerate().filter_map(|(i, o)| o.as_ref().map(|v| (i, v)))
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn capacity(&self) -> usize { self.items.len() }
}

impl<T: Clone> Default for SlotVec<T> {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// BitSet
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitSet {
    bits: Vec<u64>,
    size: usize,
}

impl BitSet {
    pub fn new(size: usize) -> Self {
        let words = size.div_ceil(64);
        BitSet { bits: vec![0; words], size }
    }

    pub fn set(&mut self, index: usize) {
        assert!(index < self.size, "BitSet::set: index out of bounds");
        self.bits[index / 64] |= 1u64 << (index % 64);
    }

    pub fn clear(&mut self, index: usize) {
        assert!(index < self.size, "BitSet::clear: index out of bounds");
        self.bits[index / 64] &= !(1u64 << (index % 64));
    }

    pub fn toggle(&mut self, index: usize) {
        assert!(index < self.size, "BitSet::toggle: index out of bounds");
        self.bits[index / 64] ^= 1u64 << (index % 64);
    }

    pub fn is_set(&self, index: usize) -> bool {
        assert!(index < self.size, "BitSet::is_set: index out of bounds");
        (self.bits[index / 64] & (1u64 << (index % 64))) != 0
    }

    pub fn count(&self) -> usize {
        self.bits.iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn union(&self, other: &BitSet) -> BitSet {
        assert_eq!(self.size, other.size, "BitSet::union: size mismatch");
        let bits: Vec<u64> = self.bits.iter().zip(other.bits.iter()).map(|(a, b)| a | b).collect();
        BitSet { bits, size: self.size }
    }

    pub fn intersection(&self, other: &BitSet) -> BitSet {
        assert_eq!(self.size, other.size, "BitSet::intersection: size mismatch");
        let bits: Vec<u64> = self.bits.iter().zip(other.bits.iter()).map(|(a, b)| a & b).collect();
        BitSet { bits, size: self.size }
    }

    pub fn difference(&self, other: &BitSet) -> BitSet {
        assert_eq!(self.size, other.size, "BitSet::difference: size mismatch");
        let bits: Vec<u64> = self.bits.iter().zip(other.bits.iter()).map(|(a, b)| a & !b).collect();
        BitSet { bits, size: self.size }
    }

    pub fn is_subset(&self, other: &BitSet) -> bool {
        assert_eq!(self.size, other.size, "BitSet::is_subset: size mismatch");
        self.bits.iter().zip(other.bits.iter()).all(|(a, b)| a & !b == 0)
    }

    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.bits.iter().enumerate().flat_map(|(wi, &word)| {
            let base = wi * 64;
            (0..64.min(self.size - base)).filter_map(move |b| {
                if (word & (1u64 << b)) != 0 { Some(base + b) } else { None }
            })
        })
    }

    pub fn size(&self) -> usize { self.size }
    pub fn is_empty(&self) -> bool { self.bits.iter().all(|&w| w == 0) }
}

impl PartialEq for BitSet {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.bits == other.bits
    }
}
impl Eq for BitSet {}

// ---------------------------------------------------------------------------
// FreeList
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FreeSlot {
    Occupied(u64, u64),
    Free { next: Option<usize>, generation: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeList {
    slots: Vec<FreeSlot>,
    first_free: Option<usize>,
}

impl FreeList {
    pub fn new() -> Self {
        FreeList { slots: Vec::new(), first_free: None }
    }

    pub fn alloc(&mut self, value: u64) -> (usize, u64) {
        match self.first_free {
            Some(idx) => {
                let gen = match &self.slots[idx] {
                    FreeSlot::Free { generation, .. } => *generation,
                    _ => unreachable!(),
                };
                let next_free = match &self.slots[idx] {
                    FreeSlot::Free { next, .. } => *next,
                    _ => unreachable!(),
                };
                self.slots[idx] = FreeSlot::Occupied(value, gen);
                self.first_free = next_free;
                (idx, gen)
            }
            None => {
                let idx = self.slots.len();
                self.slots.push(FreeSlot::Occupied(value, 0));
                (idx, 0)
            }
        }
    }

    pub fn dealloc(&mut self, index: usize, gen: u64) -> bool {
        if index >= self.slots.len() { return false; }
        match &self.slots[index] {
            FreeSlot::Occupied(_, stored_gen) if *stored_gen == gen => {
                self.slots[index] = FreeSlot::Free { next: self.first_free, generation: gen + 1 };
                self.first_free = Some(index);
                true
            }
            _ => false,
        }
    }

    pub fn get(&self, index: usize, _gen: u64) -> Option<u64> {
        match self.slots.get(index)? {
            FreeSlot::Occupied(val, _stored_gen) => Some(*val),
            _ => None,
        }
    }

    pub fn is_occupied(&self, index: usize) -> bool {
        matches!(self.slots.get(index), Some(FreeSlot::Occupied(..)))
    }

    pub fn occupied_count(&self) -> usize {
        self.slots.iter().filter(|s| matches!(s, FreeSlot::Occupied(..))).count()
    }
}

impl Default for FreeList {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// SmallVec
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum SmallVec<T, const N: usize> {
    Inline { data: [std::mem::MaybeUninit<T>; N], len: usize },
    Spilled(Vec<T>),
}

impl<T: Serialize, const N: usize> Serialize for SmallVec<T, N> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let len = self.len();
        let mut seq = serializer.serialize_seq(Some(len))?;
        for i in 0..len { seq.serialize_element(&self[i])?; }
        seq.end()
    }
}

impl<'de, T: Deserialize<'de>, const N: usize> Deserialize<'de> for SmallVec<T, N> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V<T, const M: usize>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>, const M: usize> serde::de::Visitor<'de> for V<T, M> {
            type Value = SmallVec<T, M>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a sequence")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut sv = SmallVec::new();
                while let Some(elem) = seq.next_element()? { sv.push(elem); }
                Ok(sv)
            }
        }
        deserializer.deserialize_seq(V(std::marker::PhantomData))
    }
}

impl<T, const N: usize> SmallVec<T, N> {
    pub fn new() -> Self {
        assert!(N > 0, "SmallVec requires N > 0");
        SmallVec::Inline {
            data: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        match self {
            Self::Inline { data, len } if *len < N => {
                data[*len] = std::mem::MaybeUninit::new(item);
                *len += 1;
            }
            Self::Inline { data, len } => {
                let mut vec = Vec::with_capacity(N + 1);
                for item in data.iter_mut().take(*len) { vec.push(unsafe { item.assume_init_read() }); }
                vec.push(item);
                // Clear len before assignment to prevent double-drop
                *len = 0;
                *self = Self::Spilled(vec);
            }
            Self::Spilled(vec) => { vec.push(item); }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        match self {
            Self::Inline { data, len } if *len > 0 => {
                *len -= 1;
                Some(unsafe { data[*len].assume_init_read() })
            }
            Self::Inline { .. } => None,
            Self::Spilled(vec) => vec.pop(),
        }
    }

    pub fn len(&self) -> usize {
        match self { Self::Inline { len, .. } => *len, Self::Spilled(vec) => vec.len() }
    }

    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn is_inline(&self) -> bool { matches!(self, Self::Inline { .. }) }

    pub fn get(&self, i: usize) -> Option<&T> {
        match self {
            Self::Inline { data, len } if i < *len => Some(unsafe { data[i].assume_init_ref() }),
            Self::Inline { .. } => None,
            Self::Spilled(vec) => vec.get(i),
        }
    }

    pub fn get_mut(&mut self, i: usize) -> Option<&mut T> {
        match self {
            Self::Inline { data, len } if i < *len => Some(unsafe { data[i].assume_init_mut() }),
            Self::Inline { .. } => None,
            Self::Spilled(vec) => vec.get_mut(i),
        }
    }
}

impl<T, const N: usize> Index<usize> for SmallVec<T, N> {
    type Output = T;
    fn index(&self, i: usize) -> &Self::Output {
        self.get(i).expect("SmallVec::index: index out of bounds")
    }
}

impl<T, const N: usize> Default for SmallVec<T, N> {
    fn default() -> Self { Self::new() }
}

impl<T, const N: usize> Drop for SmallVec<T, N> {
    fn drop(&mut self) {
        if let Self::Inline { data, len } = self {
            for item in data.iter_mut().take(*len) { unsafe { item.assume_init_drop(); } }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- SparseSet ----

    #[test]
    fn sparse_set_new_is_empty() {
        let s = SparseSet::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn sparse_set_insert_and_get() {
        let mut s = SparseSet::new();
        s.insert(42, 99.5);
        assert!(s.contains(42));
        assert_eq!(s.get(42), Some(99.5));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn sparse_set_overwrite() {
        let mut s = SparseSet::new();
        s.insert(10, 1.0);
        s.insert(10, 2.0);
        assert_eq!(s.get(10), Some(2.0));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn sparse_set_remove() {
        let mut s = SparseSet::new();
        s.insert(0, 1.0);
        s.insert(100, 2.0);
        assert!(s.remove(0));
        assert!(!s.contains(0));
        assert!(s.contains(100));
        assert_eq!(s.len(), 1);
        assert!(!s.remove(0));
    }

    #[test]
    fn sparse_set_remove_nonexistent() {
        let mut s = SparseSet::new();
        assert!(!s.remove(999));
    }

    #[test]
    fn sparse_set_get_mut() {
        let mut s = SparseSet::new();
        s.insert(7, 1.0);
        *s.get_mut(7).unwrap() = 99.0;
        assert_eq!(s.get(7), Some(99.0));
    }

    #[test]
    fn sparse_set_get_nonexistent() {
        let s = SparseSet::new();
        assert_eq!(s.get(0), None);
    }

    #[test]
    fn sparse_set_get_mut_nonexistent() {
        let mut s = SparseSet::new();
        assert!(s.get_mut(0).is_none());
    }

    #[test]
    fn sparse_set_iter() {
        let mut s = SparseSet::new();
        s.insert(0, 10.0);
        s.insert(1, 20.0);
        s.insert(2, 30.0);
        let mut pairs: Vec<_> = s.iter().collect();
        pairs.sort_by_key(|&(i, _)| i);
        assert_eq!(pairs, vec![(0, 10.0), (1, 20.0), (2, 30.0)]);
    }

    #[test]
    fn sparse_set_clear() {
        let mut s = SparseSet::new();
        s.insert(5, 1.0);
        s.insert(10, 2.0);
        s.clear();
        assert!(s.is_empty());
        assert!(!s.contains(5));
        assert!(!s.contains(10));
    }

    #[test]
    fn sparse_set_swap_removes_dense_correctly() {
        let mut s = SparseSet::new();
        s.insert(0, 1.0);
        s.insert(1, 2.0);
        s.insert(2, 3.0);
        s.remove(0);
        let mut items: Vec<_> = s.iter().collect();
        items.sort_by_key(|&(i, _)| i);
        assert_eq!(items.len(), 2);
        assert!(items.contains(&(1, 2.0)));
        assert!(items.contains(&(2, 3.0)));
    }

    #[test]
    fn sparse_set_serde_roundtrip() {
        let mut s = SparseSet::new();
        s.insert(0, 1.0);
        s.insert(5, 2.0);
        let json = serde_json::to_string(&s).unwrap();
        let back: SparseSet = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get(0), Some(1.0));
        assert_eq!(back.get(5), Some(2.0));
        assert_eq!(back.len(), 2);
    }

    // ---- SlotVec ----

    #[test]
    fn slotvec_new_is_empty() {
        let sv: SlotVec<i32> = SlotVec::new();
        assert!(sv.is_empty());
        assert_eq!(sv.len(), 0);
        assert_eq!(sv.capacity(), 0);
    }

    #[test]
    fn slotvec_push_and_get() {
        let mut sv = SlotVec::new();
        let i0 = sv.push(10);
        let i1 = sv.push(20);
        assert_eq!(sv.get(i0), Some(&10));
        assert_eq!(sv.get(i1), Some(&20));
        assert_eq!(sv.len(), 2);
    }

    #[test]
    fn slotvec_remove_and_reuse() {
        let mut sv = SlotVec::new();
        let _i0 = sv.push(1);
        let i1 = sv.push(2);
        let _i2 = sv.push(3);
        assert_eq!(sv.remove(i1), Some(2));
        assert_eq!(sv.len(), 2);
        let i3 = sv.push(4);
        assert_eq!(i3, i1);
        assert_eq!(sv.get(i3), Some(&4));
        assert_eq!(sv.len(), 3);
    }

    #[test]
    fn slotvec_get_mut() {
        let mut sv = SlotVec::new();
        let idx = sv.push(42);
        *sv.get_mut(idx).unwrap() = 99;
        assert_eq!(sv.get(idx), Some(&99));
    }

    #[test]
    fn slotvec_remove_nonexistent() {
        let mut sv: SlotVec<i32> = SlotVec::new();
        assert_eq!(sv.remove(0), None);
        assert_eq!(sv.remove(100), None);
    }

    #[test]
    fn slotvec_remove_twice_returns_none() {
        let mut sv = SlotVec::new();
        let idx = sv.push(10);
        assert_eq!(sv.remove(idx), Some(10));
        assert_eq!(sv.remove(idx), None);
        assert!(sv.is_empty());
    }

    #[test]
    fn slotvec_iter() {
        let mut sv = SlotVec::new();
        sv.push(1);
        sv.push(2);
        sv.push(3);
        let collected: Vec<_> = sv.iter().map(|(_, v)| *v).collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn slotvec_iter_skips_removed() {
        let mut sv = SlotVec::new();
        sv.push('a');
        let idx = sv.push('b');
        sv.push('c');
        sv.remove(idx);
        let collected: Vec<_> = sv.iter().map(|(_, v)| *v).collect();
        assert_eq!(collected, vec!['a', 'c']);
    }

    #[test]
    fn slotvec_serde_roundtrip() {
        let mut sv = SlotVec::new();
        sv.push("hello".to_string());
        sv.push("world".to_string());
        let json = serde_json::to_string(&sv).unwrap();
        let back: SlotVec<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back.get(0).map(|s| s.as_str()), Some("hello"));
    }

    // ---- BitSet ----

    #[test]
    fn bitset_new_all_clear() {
        let bs = BitSet::new(100);
        assert_eq!(bs.count(), 0);
        assert!(bs.is_empty());
    }

    #[test]
    fn bitset_set_and_is_set() {
        let mut bs = BitSet::new(64);
        bs.set(0);
        bs.set(63);
        assert!(bs.is_set(0));
        assert!(bs.is_set(63));
        assert!(!bs.is_set(1));
        assert_eq!(bs.count(), 2);
    }

    #[test]
    fn bitset_clear() {
        let mut bs = BitSet::new(64);
        bs.set(10);
        bs.clear(10);
        assert!(!bs.is_set(10));
    }

    #[test]
    fn bitset_toggle() {
        let mut bs = BitSet::new(64);
        assert!(!bs.is_set(5));
        bs.toggle(5);
        assert!(bs.is_set(5));
        bs.toggle(5);
        assert!(!bs.is_set(5));
    }

    #[test]
    fn bitset_count_multiword() {
        let mut bs = BitSet::new(128);
        bs.set(0);
        bs.set(64);
        bs.set(127);
        assert_eq!(bs.count(), 3);
    }

    #[test]
    fn bitset_union() {
        let mut a = BitSet::new(64);
        let mut b = BitSet::new(64);
        a.set(0);
        b.set(1);
        let u = a.union(&b);
        assert!(u.is_set(0));
        assert!(u.is_set(1));
        assert_eq!(u.count(), 2);
    }

    #[test]
    fn bitset_intersection() {
        let mut a = BitSet::new(64);
        let mut b = BitSet::new(64);
        a.set(0);
        a.set(1);
        b.set(0);
        let i = a.intersection(&b);
        assert!(i.is_set(0));
        assert!(!i.is_set(1));
    }

    #[test]
    fn bitset_difference() {
        let mut a = BitSet::new(64);
        let mut b = BitSet::new(64);
        a.set(0);
        a.set(1);
        b.set(0);
        let d = a.difference(&b);
        assert!(!d.is_set(0));
        assert!(d.is_set(1));
    }

    #[test]
    fn bitset_is_subset() {
        let mut a = BitSet::new(64);
        let mut b = BitSet::new(64);
        a.set(5);
        b.set(5);
        b.set(10);
        assert!(a.is_subset(&b));
        assert!(!b.is_subset(&a));
    }

    #[test]
    fn bitset_equal_sets_are_subsets() {
        let mut a = BitSet::new(64);
        let mut b = BitSet::new(64);
        a.set(3);
        a.set(7);
        b.set(3);
        b.set(7);
        assert!(a.is_subset(&b));
        assert!(b.is_subset(&a));
    }

    #[test]
    fn bitset_iter() {
        let mut bs = BitSet::new(200);
        bs.set(0);
        bs.set(1);
        bs.set(127);
        bs.set(128);
        bs.set(199);
        let indices: Vec<_> = bs.iter().collect();
        assert_eq!(indices, vec![0, 1, 127, 128, 199]);
    }

    #[test]
    fn bitset_iter_empty() {
        let bs = BitSet::new(64);
        let indices: Vec<_> = bs.iter().collect();
        assert!(indices.is_empty());
    }

    #[test]
    fn bitset_serde_roundtrip() {
        let mut bs = BitSet::new(128);
        bs.set(0);
        bs.set(64);
        let json = serde_json::to_string(&bs).unwrap();
        let back: BitSet = serde_json::from_str(&json).unwrap();
        assert_eq!(back.size(), 128);
        assert!(back.is_set(0));
        assert!(back.is_set(64));
        assert!(!back.is_set(1));
        assert_eq!(back.count(), 2);
    }

    // ---- FreeList ----

    #[test]
    fn freelist_alloc_and_get() {
        let mut fl = FreeList::new();
        let (idx, gen) = fl.alloc(42);
        assert_eq!(fl.get(idx, gen), Some(42));
        assert!(fl.is_occupied(idx));
        assert_eq!(fl.occupied_count(), 1);
    }

    #[test]
    fn freelist_dealloc_and_reuse() {
        let mut fl = FreeList::new();
        let (idx, gen) = fl.alloc(42);
        assert!(fl.dealloc(idx, gen));
        assert!(!fl.is_occupied(idx));
        assert_eq!(fl.occupied_count(), 0);
        let (idx2, gen2) = fl.alloc(99);
        assert_eq!(idx2, idx);
        assert_eq!(gen2, gen + 1);
        assert_eq!(fl.get(idx2, gen2), Some(99));
        assert_eq!(fl.occupied_count(), 1);
    }

    #[test]
    fn freelist_dealloc_wrong_generation() {
        let mut fl = FreeList::new();
        let (idx, _gen) = fl.alloc(42);
        assert!(!fl.dealloc(idx, 999));
        assert!(fl.is_occupied(idx));
    }

    #[test]
    fn freelist_out_of_bounds() {
        let mut fl: FreeList = FreeList::new();
        assert!(!fl.is_occupied(0));
        assert!(!fl.dealloc(0, 0));
        assert_eq!(fl.get(0, 0), None);
    }

    #[test]
    fn freelist_serde_roundtrip() {
        let mut fl = FreeList::new();
        let (idx, gen) = fl.alloc(7);
        let json = serde_json::to_string(&fl).unwrap();
        let back: FreeList = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get(idx, gen), Some(7));
        assert_eq!(back.occupied_count(), 1);
    }

    // ---- SmallVec ----

    #[test]
    fn smallvec_new_is_inline_and_empty() {
        let sv: SmallVec<i32, 4> = SmallVec::new();
        assert!(sv.is_inline());
        assert!(sv.is_empty());
        assert_eq!(sv.len(), 0);
    }

    #[test]
    fn smallvec_push_inline() {
        let mut sv: SmallVec<i32, 4> = SmallVec::new();
        sv.push(10);
        sv.push(20);
        assert!(sv.is_inline());
        assert_eq!(sv.len(), 2);
        assert_eq!(sv[0], 10);
        assert_eq!(sv[1], 20);
    }

    #[test]
    fn smallvec_pop() {
        let mut sv: SmallVec<i32, 3> = SmallVec::new();
        sv.push(1);
        sv.push(2);
        sv.push(3);
        assert_eq!(sv.pop(), Some(3));
        assert_eq!(sv.len(), 2);
        assert_eq!(sv.pop(), Some(2));
        assert_eq!(sv.pop(), Some(1));
        assert_eq!(sv.pop(), None);
    }

    #[test]
    fn smallvec_push_spills() {
        let mut sv: SmallVec<i32, 2> = SmallVec::new();
        sv.push(1);
        sv.push(2);
        assert!(sv.is_inline());
        sv.push(3);
        assert!(!sv.is_inline());
        assert_eq!(sv.len(), 3);
        assert_eq!(sv[0], 1);
        assert_eq!(sv[1], 2);
        assert_eq!(sv[2], 3);
    }

    #[test]
    fn smallvec_get_and_get_mut() {
        let mut sv: SmallVec<i32, 4> = SmallVec::new();
        sv.push(10);
        sv.push(20);
        assert_eq!(sv.get(0), Some(&10));
        assert_eq!(sv.get(5), None);
        *sv.get_mut(0).unwrap() = 99;
        assert_eq!(sv[0], 99);
    }

    #[test]
    fn smallvec_index_trait() {
        let mut sv: SmallVec<i32, 4> = SmallVec::new();
        sv.push(42);
        assert_eq!(sv[0], 42);
    }

    #[test]
    fn smallvec_default() {
        let sv: SmallVec<u32, 8> = SmallVec::default();
        assert!(sv.is_empty());
    }

    #[test]
    fn smallvec_serde_inline() {
        let mut sv: SmallVec<i32, 4> = SmallVec::new();
        sv.push(1);
        sv.push(2);
        let json = serde_json::to_string(&sv).unwrap();
        let back: SmallVec<i32, 4> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back[0], 1);
        assert_eq!(back[1], 2);
        assert!(back.is_inline());
    }

    #[test]
    fn smallvec_serde_spilled() {
        let mut sv: SmallVec<String, 2> = SmallVec::new();
        sv.push("a".into());
        sv.push("b".into());
        sv.push("c".into());
        let json = serde_json::to_string(&sv).unwrap();
        let back: SmallVec<String, 2> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 3);
        assert_eq!(back[0], "a");
        assert_eq!(back[1], "b");
        assert_eq!(back[2], "c");
        assert!(!back.is_inline());
    }

    #[test]
    #[should_panic(expected = "SmallVec::index: index out of bounds")]
    fn smallvec_index_out_of_bounds() {
        let sv: SmallVec<i32, 4> = SmallVec::new();
        let _ = sv[0];
    }
}
