use rustc_hash::{FxBuildHasher, FxHashSet};
use std::collections::{HashSet, VecDeque};
use tracing::{debug, trace};

/// A ring buffer that also keeps track of the keys it contains to avoid duplicates.
///
/// This serves as a backend for the preview cache.
/// Basic idea:
/// - When a new key is pushed, if it's already in the buffer, do nothing.
/// - If the buffer is full, remove the oldest key and push the new key.
///
/// # Example
/// ```rust
/// use television::utils::cache::RingSet;
///
/// let mut ring_set = RingSet::with_capacity(3);
/// // push 3 values into the ringset
/// assert_eq!(ring_set.push(1), None);
/// assert_eq!(ring_set.push(2), None);
/// assert_eq!(ring_set.push(3), None);
///
/// // check that the values are in the buffer
/// assert!(ring_set.contains(&1));
/// assert!(ring_set.contains(&2));
/// assert!(ring_set.contains(&3));
///
/// // push an existing value (should do nothing)
/// assert_eq!(ring_set.push(1), None);
///
/// // entries should still be there
/// assert!(ring_set.contains(&1));
/// assert!(ring_set.contains(&2));
/// assert!(ring_set.contains(&3));
///
/// // push a new value, should remove the oldest value (1)
/// assert_eq!(ring_set.push(4), Some(1));
///
/// // 1 is no longer there but 2 and 3 remain
/// assert!(!ring_set.contains(&1));
/// assert!(ring_set.contains(&2));
/// assert!(ring_set.contains(&3));
/// assert!(ring_set.contains(&4));
/// ```
#[derive(Debug)]
pub struct RingSet<T> {
    ring_buffer: VecDeque<T>,
    known_keys: FxHashSet<T>,
    capacity: usize,
}

const DEFAULT_CAPACITY: usize = 20;

impl<T> Default for RingSet<T>
where
    T: Eq + std::hash::Hash + Clone + std::fmt::Debug,
{
    fn default() -> Self {
        RingSet::with_capacity(DEFAULT_CAPACITY)
    }
}

impl<T> RingSet<T>
where
    T: Eq + std::hash::Hash + Clone + std::fmt::Debug,
{
    /// Create a new `RingSet` with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        RingSet {
            ring_buffer: VecDeque::with_capacity(capacity),
            known_keys: HashSet::with_capacity_and_hasher(
                capacity,
                FxBuildHasher,
            ),
            capacity,
        }
    }

    /// Push a new item to the back of the buffer, removing the oldest item if the buffer is full.
    /// Returns the item that was removed, if any.
    /// If the item is already in the buffer, do nothing and return None.
    pub fn push(&mut self, item: T) -> Option<T> {
        // If the key is already in the buffer, do nothing
        if self.contains(&item) {
            trace!("Key already in ring buffer: {:?}", item);
            return None;
        }
        let mut popped_key = None;
        // If the buffer is full, remove the oldest key (e.g. pop from the front of the buffer)
        if self.ring_buffer.len() >= self.capacity {
            popped_key = self.pop();
        }
        // finally, push the new key to the back of the buffer
        self.ring_buffer.push_back(item.clone());
        self.known_keys.insert(item);
        popped_key
    }

    fn pop(&mut self) -> Option<T> {
        if let Some(item) = self.ring_buffer.pop_front() {
            debug!("Removing key from ring buffer: {:?}", item);
            self.known_keys.remove(&item);
            Some(item)
        } else {
            None
        }
    }

    pub fn contains(&self, key: &T) -> bool {
        self.known_keys.contains(key)
    }

    /// Returns an iterator that goes from the back to the front of the buffer.
    pub fn back_to_front(&self) -> impl Iterator<Item = T> {
        self.ring_buffer.clone().into_iter().rev()
    }

    /// Returns the current size of the ring buffer, which is the number of unique keys it
    /// contains.
    pub fn size(&self) -> usize {
        self.known_keys.len()
    }

    /// Wipes the ring buffer clean.
    pub fn clear(&mut self) {
        debug!("Clearing ring buffer");
        self.ring_buffer.clear();
        self.known_keys.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_set() {
        let mut ring_set = RingSet::with_capacity(3);
        // push 3 values into the ringset
        assert_eq!(ring_set.push(1), None);
        assert_eq!(ring_set.push(2), None);
        assert_eq!(ring_set.push(3), None);

        // check that the values are in the buffer
        assert!(ring_set.contains(&1));
        assert!(ring_set.contains(&2));
        assert!(ring_set.contains(&3));

        // push an existing value (should do nothing)
        assert_eq!(ring_set.push(1), None);

        // entries should still be there
        assert!(ring_set.contains(&1));
        assert!(ring_set.contains(&2));
        assert!(ring_set.contains(&3));

        // push a new value, should remove the oldest value (1)
        assert_eq!(ring_set.push(4), Some(1));

        // 1 is no longer there but 2 and 3 remain
        assert!(!ring_set.contains(&1));
        assert!(ring_set.contains(&2));
        assert!(ring_set.contains(&3));
        assert!(ring_set.contains(&4));

        // push two new values, should remove 2 and 3
        assert_eq!(ring_set.push(5), Some(2));
        assert_eq!(ring_set.push(6), Some(3));

        // 2 and 3 are no longer there but 4, 5 and 6 remain
        assert!(!ring_set.contains(&2));
        assert!(!ring_set.contains(&3));
        assert!(ring_set.contains(&4));
        assert!(ring_set.contains(&5));
        assert!(ring_set.contains(&6));
    }
}
