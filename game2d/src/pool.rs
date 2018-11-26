use std::hash::Hash;
use std::hash::Hasher;
use std::mem;

const MATCH_ALL_IDS: u32 = 0;
const FIRST_VALID_ID: u32 = 1;

enum Entry<T> {
    /// `usize` parameter indicates next free slot
    Free(usize),
    /// `usize` parameter used for unique ID
    Value(u32, T),
}

/// A `Pool` is pre-allocated array that can be used for managing a collection of objects. Unlike a
/// standard vector, a pool is optimized to avoid fragmentation - when you remove an object from the
/// middle of the list, it is marked re-usable - all previously allocated objects don't move, and
/// the next allocation request will be given that spot.
///
/// Don't use a pool if you want a collection that can shrink / reclaim memory over time, or if you
/// need the insertion order to matter.
pub struct Pool<T> {
    entries: Vec<Entry<T>>,
    next_free: usize,
    len: usize,
    next_id: u32,
}

/// A handle will be returned to the caller by the pool when they add a new object, and it can then
/// be used to safely query / remove the object later.
#[derive(Debug, Clone, Copy)]
pub struct Handle {
    index: usize,
    /// ID which verifies that the entry we fetched by this handle is actually the one the handle
    /// was originally associated with (vs. the old entry being removed and a new entry being
    /// allocated into its spot later).
    entry_id: u32,
}

impl Eq for Handle {}
impl PartialEq<Handle> for Handle {
    fn eq(&self, other: &Handle) -> bool {
        self.entry_id.eq(&other.entry_id) // ID alone guarantees equality
    }
}
impl Hash for Handle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.entry_id); // ID alone guarantees uniqueness
    }
}

impl<T> Entry<T> {
    /// Returns `true` if this is an `Entry::Free`, indicating this slot is open for storing a
    /// value.
    #[inline]
    pub fn is_free(&self) -> bool {
        match *self {
            Entry::Free(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if this is an `Entry::Value`, indicating this slot contains a value.
    #[inline]
    pub fn has_value(&self) -> bool {
        !self.is_free()
    }

    /// Returns the ID of this entry, if this is an `Entry::Value`.
    ///
    /// The ID helps protect access from stale `Handle`s that points to an index since recycled.
    #[inline]
    pub fn id(&self) -> Option<u32> {
        if let Entry::Value(id, _) = self {
            return Some(*id);
        }
        None
    }

    /// Return the wrapped value of this entry, if this is an `Entry::Value`
    #[inline]
    pub fn value(&self) -> Option<&T> {
        self.value_with_id(MATCH_ALL_IDS)
    }

    /// Mutable version of `value`
    #[inline]
    pub fn value_mut(&mut self) -> Option<&mut T> {
        self.value_with_id_mut(MATCH_ALL_IDS)
    }

    /// Return the wrapped value of this entry, if this is an `Entry::Value` and if `id` matches the
    /// entry's ID. This method can help protect access from stale `Handle`s that points to an
    /// index since recycled.
    #[inline]
    pub fn value_with_id(&self, id: u32) -> Option<&T> {
        if let Entry::Value(value_id, value) = self {
            if id == MATCH_ALL_IDS || *value_id == id {
                return Some(&*value);
            }
        }
        None
    }

    /// Mutable version of `value_with_id`
    #[inline]
    pub fn value_with_id_mut(&mut self, id: u32) -> Option<&mut T> {
        if let Entry::Value(value_id, value) = self {
            if id == MATCH_ALL_IDS || *value_id == id {
                return Some(&mut *value);
            }
        }
        None
    }
}

#[allow(clippy::new_without_default_derive)] // API is intentionally explicit
impl<T> Pool<T> {
    /// Create a new pool with a default capacity
    pub fn new() -> Pool<T> {
        Pool::<T>::with_capacity(10)
    }

    /// Create a new pool with an explicit capacity. It is an error to create a pool with a capacity
    /// of 0.
    pub fn with_capacity(capacity: usize) -> Pool<T> {
        if capacity == 0 {
            panic!("Can't create a pool with a capacity of 0")
        }

        let mut entries = Vec::with_capacity(capacity);
        Pool::fill_with_free_entries(&mut entries);
        Pool {
            entries,
            next_free: 0,
            len: 0,
            next_id: FIRST_VALID_ID,
        }
    }

    /// Helper function that initializes the `entries` array with `Free` items pointing at the next
    /// free slot.
    fn fill_with_free_entries(entries: &mut Vec<Entry<T>>) {
        for i in entries.len()..entries.capacity() {
            entries.push(Entry::Free(i + 1))
        }
    }

    /// The total allocated size of the pool. The capacity will automatically
    #[inline]
    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    /// Return the number of objects that have been added to this pool so far, not to be confused
    /// with the pool's `capacity`.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Add a new object to the next open free slot in this pool.
    pub fn push(&mut self, value: T) -> Handle {
        if self.len == self.entries.capacity() {
            self.entries.reserve(self.len * 2);
            Pool::fill_with_free_entries(&mut self.entries);
        }

        let next_id = self.next_id;
        self.next_id += 1;
        self.len += 1;

        let handle = Handle {
            index: self.next_free,
            entry_id: next_id,
        };

        let free_entry = mem::replace(
            &mut self.entries[self.next_free],
            Entry::Value(next_id, value),
        );
        if let Entry::Free(next_free) = free_entry {
            self.next_free = next_free;
        } else {
            panic!("Unexpected pool state: self.next_free pointed to non-free slot")
        }

        handle
    }

    /// Remove an object by its handle. This will return `None` if the object allocated for that
    /// handle was already removed.
    pub fn remove(&mut self, handle: Handle) -> Option<T> {
        // If the entry is already removed OR if a new one was reallocated in its place from the
        // object referenced by the handle, then reject this request to remove, returning None.
        self.entries[handle.index].value_with_id(handle.entry_id)?;

        let removed = mem::replace(&mut self.entries[handle.index], Entry::Free(self.next_free));
        if let Entry::Value(_, value) = removed {
            self.len -= 1;
            self.next_free = handle.index;
            return Some(value);
        } else {
            panic!("Unexpected pool state: removed entry should always be a Value");
        }
    }

    /// Query an object by its handle. This will return `None` if the object allocated for that
    /// handle was already removed.
    pub fn get(&self, handle: Handle) -> Option<&T> {
        self.entries[handle.index].value_with_id(handle.entry_id)
    }

    /// Mutable version of `get`.
    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        self.entries[handle.index].value_with_id_mut(handle.entry_id)
    }

    /// Return an iterator that provides access to all entries in this pool. The order is not
    /// guaranteed to match insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries
            .iter()
            .filter(|entry| entry.has_value())
            .map(|entry| entry.value().unwrap())
    }

    /// Mutable version of `iter`
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries
            .iter_mut()
            .filter(|entry| entry.has_value())
            .map(|entry| entry.value_mut().unwrap())
    }

    /// Return a newly collection of all handles that currently point to valid entries in this pool. It
    /// can be useful to call this instead of `iter` or `iter_mut` since those methods keep a
    /// reference to the pool while this call does not.
    pub fn handles(&self) -> impl Iterator<Item = Handle> {
        self.entries
            .iter()
            .enumerate() // We need the index to create handles
            .filter(|(_i, entry)| entry.has_value())
            .map(|(i, entry)| Handle {
                index: i,
                entry_id: entry.id().unwrap(),
            })
            // Up to this point, the iterator keeps a reference to self.entries. We want to break
            // that link, so we do it by creating a new vector and returning that as an iterator.
            // (There may be a better way to do this but it seems to work for now!)
            .collect::<Vec<Handle>>()
            .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_impl_methods_work() {
        let mut free_entry: Entry<&str> = Entry::Free(20);
        let mut value_entry: Entry<&str> = Entry::Value(5, "test");

        assert_eq!(free_entry.is_free(), true);
        assert_eq!(free_entry.has_value(), false);
        assert_eq!(free_entry.value().is_none(), true);
        assert_eq!(free_entry.value_mut().is_none(), true);
        assert_eq!(free_entry.value_with_id(1).is_none(), true);
        assert_eq!(free_entry.value_with_id_mut(1).is_none(), true);
        assert_eq!(free_entry.value_with_id(5).is_none(), true);
        assert_eq!(free_entry.value_with_id_mut(5).is_none(), true);

        assert_eq!(value_entry.is_free(), false);
        assert_eq!(value_entry.has_value(), true);
        assert_eq!(value_entry.value().unwrap(), &"test");
        assert_eq!(value_entry.value_mut().unwrap(), &"test");
        assert_eq!(value_entry.value_with_id(1).is_none(), true);
        assert_eq!(value_entry.value_with_id_mut(1).is_none(), true);
        assert_eq!(value_entry.value_with_id(5).unwrap(), &"test");
        assert_eq!(value_entry.value_with_id_mut(5).unwrap(), &"test");
    }
}
