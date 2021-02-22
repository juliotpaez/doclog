use std::collections::BTreeMap;
use std::ops::{Bound, Range};

use range_wrapper::RangeStartWrapper;

mod range_wrapper;

// Based on: https://github.com/jeffparsons/rangemap/

#[derive(Debug, Clone)]
pub struct RangeMap<V> {
    btree: BTreeMap<RangeStartWrapper, V>,
}

impl<V> RangeMap<V> {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new() -> Self {
        RangeMap {
            btree: BTreeMap::new(),
        }
    }

    // GETTERS ----------------------------------------------------------------

    pub fn is_empty(&self) -> bool {
        self.btree.is_empty()
    }

    // METHODS ----------------------------------------------------------------

    /// Returns `true` if any range in the map collides with `range`.
    pub fn collides_with(&self, range: Range<usize>) -> bool {
        let new_range_wrapper = RangeStartWrapper::new(range);

        // Compare to previous.
        if let Some((range_wrapper, _)) = self
            .btree
            .range((Bound::Unbounded, Bound::Included(&new_range_wrapper)))
            .next_back()
        {
            if range_wrapper.overlaps(&new_range_wrapper) {
                return false;
            }
        }

        // Compare to next.
        if let Some((range_wrapper, _)) = self
            .btree
            .range((Bound::Excluded(&new_range_wrapper), Bound::Unbounded))
            .next_back()
        {
            if range_wrapper.overlaps(&new_range_wrapper) {
                return false;
            }
        }

        true
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Range<usize>, &V)> {
        self.btree.iter().map(|(by_start, v)| (&by_start.range, v))
    }

    pub fn insert(&mut self, range: Range<usize>, value: V) {
        self.btree.insert(RangeStartWrapper::new(range), value);
    }
}
