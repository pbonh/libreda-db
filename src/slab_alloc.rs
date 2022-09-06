// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Container with `O(1)` insertion, lookup and remove operations.

#![allow(unused)]

use num_traits::{FromPrimitive, PrimInt, ToPrimitive, Unsigned};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct Element<IdType, T> {
    id: IdType,
    value: Option<T>,
}

/// Slab allocator with 8-bit indices. Can hold at most 2^8 elements.
pub type SlabAlloc8<T> = SlabAlloc<T, u8, u8>;
/// Slab allocator with 16-bit indices. Can hold at most 2^16 elements.
pub type SlabAlloc16<T> = SlabAlloc<T, u16, u16>;
/// Slab allocator with 32-bit indices. Can hold at most 2^32 elements.
pub type SlabAlloc32<T> = SlabAlloc<T, u32, u32>;
/// Slab allocator with 64-bit indices. Can hold at most 2^64 elements.
pub type SlabAlloc64<T> = SlabAlloc<T, u64, u64>;
/// Slab allocator with n-bit indices. Can hold at most 2^n elements.
/// Where `n` is the machine word size.
pub type SlabAllocUsize<T> = SlabAlloc<T, usize, usize>;

pub type SlabIndex8 = SlabIndex<u8, u8>;
pub type SlabIndex16 = SlabIndex<u16, u16>;
pub type SlabIndex32 = SlabIndex<u32, u32>;
pub type SlabIndex64 = SlabIndex<u64, u64>;
pub type SlabIndexUsize = SlabIndex<usize, usize>;

/// Container which efficiently allocates space for its elements.
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SlabAlloc<T, IndexType = u32, IdType = u32> {
    data: Vec<Element<IdType, T>>,
    free_indices: Vec<IndexType>,
    /// Number of elements currently in the map.
    len: usize,
}

/// Key into an [`SlabAlloc`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SlabIndex<IndexType, IdType> {
    /// Pointer into the data array.
    index: IndexType,
    /// A monotonically increasing ID which makes sure that the same index is not repeated.
    id: IdType,
}

impl<IndexType, IdType> SlabIndex<IndexType, IdType>
    where IndexType: PrimInt + Unsigned + ToPrimitive {
    fn index(&self) -> usize {
        self.index.to_usize().unwrap()
    }
}

#[allow(unused)]
impl<T, IndexType, IdType> SlabAlloc<T, IndexType, IdType>
    where IndexType: PrimInt + Unsigned + ToPrimitive + FromPrimitive,
          IdType: PrimInt + Unsigned {
    /// Create an empty container.
    pub fn new() -> Self {
        Self {
            free_indices: vec![],
            data: vec![],
            len: 0,
        }
    }

    /// Access an element.
    pub fn get(&self, index: SlabIndex<IndexType, IdType>) -> Option<&T> {
        if let Some(entry) = self.data.get(index.index()) {
            if entry.id == index.id {
                entry.value.as_ref()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Access an element.
    pub fn get_mut(&mut self, index: SlabIndex<IndexType, IdType>) -> Option<&mut T> {
        if let Some(entry) = self.data.get_mut(index.index()) {
            if entry.id == index.id {
                entry.value.as_mut()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn contains_key(&self, index: SlabIndex<IndexType, IdType>) -> bool {
        self.get(index).is_some()
    }

    /// Insert an element and return the index of it.
    ///
    /// # Panics
    /// Panics when the amount of indices is exhausted. This happens
    /// when there are already `IndexType::max_value()` elements in the container.
    pub fn insert(&mut self, value: T) -> SlabIndex<IndexType, IdType> {
        // Find a new free index.
        let index = self.free_indices.pop()
            .unwrap_or_else(|| {
                let idx = IndexType::from_usize(self.data.len())
                    .expect("slab allocator: out of indices");
                self.data.push(Element {
                    id: IdType::zero(),
                    value: None,
                }); // Extend by one element.
                idx
            });

        let index_usize = index.to_usize().unwrap();
        debug_assert!(self.data[index_usize].value.is_none());

        let entry = self.data.get_mut(index_usize).unwrap();
        let id = entry.id;

        entry.value = Some(value);
        self.len += 1;

        SlabIndex {
            index,
            id,
        }
    }

    /// Allocate space to hold `n` elements.
    pub fn reserve(&mut self, n: usize) {
        self.data.reserve(n);
    }

    /// Remove an element.
    pub fn remove(&mut self, index: SlabIndex<IndexType, IdType>) -> Option<T> {
        let entry = self.data.get_mut(index.index())?;

        if entry.id == index.id {
            // ID matches.
            let old_value = entry.value.take()?;

            self.free_indices.push(index.index);
            self.len -= 1;

            // Make sure the next allocation uses a fresh ID value for this index.
            entry.id = if entry.id < IdType::max_value() {
                entry.id + IdType::one()
            } else {
                IdType::zero() // wrapping addition
            };

            Some(old_value)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Reclaim as much space as possible.
    pub fn shrink(&mut self) {
        while self.data.last().map(|entry| entry.value.is_none()).unwrap_or(false) {
            self.data.pop();
        }

        self.free_indices.retain(|idx| idx.to_usize().unwrap() < self.data.len());
    }

    /// Drop all data.
    pub fn clear(&mut self) {
        self.data.clear();
        self.free_indices.clear();
        self.len = 0;
    }

    /// Iterate over all key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item=(SlabIndex<IndexType, IdType>, &T)> {
        self.data.iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                let index = IndexType::from_usize(index).unwrap();
                entry.value.as_ref()
                    .map(|v| (SlabIndex { index, id: entry.id }, v))
            })
    }

    /// Iterate over all key-value pairs.
    pub fn iter_mut(&mut self) -> impl Iterator<Item=(SlabIndex<IndexType, IdType>, &mut T)> {
        self.data.iter_mut()
            .enumerate()
            .filter_map(|(index, entry)| {
                entry.value.as_mut()
                    .map(|v| {
                        let index = IndexType::from_usize(index).unwrap();
                        (SlabIndex {
                            index,
                            id: entry.id,
                        }, v)
                    })
            })
    }

    /// Iterate over all values in the map.
    pub fn values(&self) -> impl Iterator<Item=&T> {
        self.data.iter()
            .filter_map(|entry| {
                entry.value.as_ref()
            })
    }

    /// Iterate over all mutable values in the map.
    pub fn values_mut(&mut self) -> impl Iterator<Item=&mut T> {
        self.data.iter_mut()
            .filter_map(|entry| {
                entry.value.as_mut()
            })
    }

    /// Iterate over all keys in the map.
    pub fn keys(&self) -> impl Iterator<Item=SlabIndex<IndexType, IdType>> + '_ {
        self.data.iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                entry.value.as_ref()
                    .map(|_| {
                        let index = IndexType::from_usize(index).unwrap();
                        SlabIndex { index, id: entry.id }
                    })
            })
    }
}


#[test]
fn test_slab_allocator() {
    let mut map = SlabAlloc32::new();

    let h1 = map.insert(1);
    let h2 = map.insert(2);

    assert_eq!(map.get(h1), Some(&1));
    assert_eq!(map.get(h2), Some(&2));

    assert_eq!(map.remove(h1), Some(1));

    let h3 = map.insert(3);
    assert_eq!(map.data.len(), 2, "position of 1 was reused");
    assert_eq!(map.get(h1), None);
    assert_eq!(map.get(h3), Some(&3));

    assert_eq!(map.iter().count(), 2);
}
