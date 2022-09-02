// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Container with `O(1)` insertion, lookup and remove operations.

/// Integer type used to make handles unique.
type IdType = u32;
/// Integer type used as pointer into the data array.
type IndexType = u32;

/// Map-like container which does not let the caller choose the key but generates it
/// when an element is inserted.
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IndexMap<T> {
    data: Vec<Option<(IdType, T)>>,
    id_counter: IdType,
    free_indices: Vec<IndexType>,
    /// Number of elements currently in the map.
    len: usize,
}

/// Key into an [`IndexMap`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Index {
    /// Pointer into the data array.
    index: IndexType,
    /// A monotonically increasing ID which makes sure that the same index is not repeated.
    id: IdType,
}

impl Index {
    fn index(&self) -> usize {
        self.index as usize
    }
}

#[allow(unused)]
impl<T> IndexMap<T> {
    /// Create an empty container.
    pub fn new() -> Self {
        Self {
            id_counter: 0,
            free_indices: vec![],
            data: vec![],
            len: 0,
        }
    }

    /// Access an element.
    pub fn get(&self, index: Index) -> Option<&T> {
        let data: &(_, _) = self.data.get(index.index())
            .and_then(|o| o.as_ref())?;

        if data.0 == index.id {
            Some(&data.1)
        } else {
            None
        }
    }

    pub fn contains_key(&self, index: Index) -> bool {
        self.get(index).is_some()
    }

    /// Insert an element and return the index of it.
    pub fn insert(&mut self, value: T) -> Index {
        // Find a new free index.
        let index = self.free_indices.pop()
            .unwrap_or_else(|| {
                let idx = self.data.len() as IndexType;
                self.data.push(None); // Extend by one element.
                idx
            });

        let id = self.id_counter;
        self.id_counter += 1;

        debug_assert!(self.data[index as usize].is_none());
        self.data[index as usize] = Some((id, value));
        self.len += 1;

        Index {
            index: index,
            id,
        }
    }

    /// Allocate space to hold `n` elements.
    pub fn reserve(&mut self, n: usize) {
        self.data.reserve(n);
    }

    /// Remove an element.
    pub fn remove(&mut self, index: Index) -> Option<T> {
        let data_id = self.data.get(index.index())
            .and_then(|o| o.as_ref())?.0;

        if data_id == index.id {
            // ID matches.
            let (_, value) = self.data.get_mut(index.index())
                .and_then(|o| o.take())?;

            self.free_indices.push(index.index);
            self.len -= 1;

            Some(value)
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
        while self.data.last().map(|o| o.is_none()).unwrap_or(false) {
            self.data.pop();
        }

        self.free_indices.retain(|idx| (*idx as usize) < self.data.len());
    }

    /// Drop all data.
    pub fn clear(&mut self) {
        self.data.clear();
        self.free_indices.clear();
        self.len = 0;
        self.id_counter = 0;
    }

    /// Iterate over all key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item=(Index, &T)> {
        self.data.iter()
            .enumerate()
            .filter_map(|(index, value)| {
                value.as_ref().map(|(id, v)| (Index {index: index as IndexType, id: *id}, v))
            })
    }

    /// Iterate over all key-value pairs.
    pub fn iter_mut(&mut self) -> impl Iterator<Item=(Index, &mut T)> {
        self.data.iter_mut()
            .enumerate()
            .filter_map(|(index, value)| {
                value.as_mut().map(|(id, v)| (Index {index: index as IndexType, id: *id}, v))
            })
    }

    /// Iterate over all values in the map.
    pub fn values(&self) ->  impl Iterator<Item=&T> {
        self.data.iter()
            .filter_map(|value| {
                value.as_ref().map(|(_, v)| v)
            })
    }

    /// Iterate over all mutable values in the map.
    pub fn values_mut(&mut self) ->  impl Iterator<Item=&mut T> {
        self.data.iter_mut()
            .filter_map(|value| {
                value.as_mut().map(|(_, v)| v)
            })
    }

    /// Iterate over all keys in the map.
    pub fn keys(&self)->  impl Iterator<Item=Index> + '_ {
        self.data.iter()
            .enumerate()
            .filter_map(|(index, value)| {
                value.as_ref().map(|(id, _)| Index {index: index as IndexType, id: *id})
            })
    }
}


#[test]
fn test_index_map() {
    let mut map = IndexMap::new();

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
