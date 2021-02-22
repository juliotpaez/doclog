use std::cmp::{max, min, Ordering};
use std::ops::Range;

#[derive(Eq, Debug, Clone)]
pub struct RangeStartWrapper {
    pub range: Range<usize>,
}

impl RangeStartWrapper {
    // CONSTRUCTORS -----------------------------------------------------------

    pub fn new(range: Range<usize>) -> RangeStartWrapper {
        RangeStartWrapper { range }
    }

    // METHODS ----------------------------------------------------------------

    pub fn overlaps(&self, other: &Self) -> bool {
        max(&self.range.start, &other.range.start) < min(&self.range.end, &other.range.end)
    }
}

impl PartialEq for RangeStartWrapper {
    fn eq(&self, other: &RangeStartWrapper) -> bool {
        self.range.start == other.range.start && self.range.len() == other.range.len()
    }
}

impl Ord for RangeStartWrapper {
    fn cmp(&self, other: &RangeStartWrapper) -> Ordering {
        match self.range.start.cmp(&other.range.start) {
            Ordering::Equal => self.range.len().cmp(&other.range.len()),
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        }
    }
}

impl PartialOrd for RangeStartWrapper {
    fn partial_cmp(&self, other: &RangeStartWrapper) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
