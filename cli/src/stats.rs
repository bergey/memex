use std::collections::{HashMap, HashSet};

use crate::tag::TagId;

pub struct TagCounts {
    counts: HashMap<TagId, u64>,
}

impl TagCounts {
    pub fn new() -> Self {
        TagCounts {
            counts: HashMap::new(),
        }
    }

    pub fn count(&mut self, tags: &HashSet<TagId>) {
        for t in tags {
            self.counts
                .entry(*t)
                .and_modify(|counter| *counter += 1)
                .or_insert(1);
        }
    }

    pub fn to_vec(self) -> Vec<(TagId, u64)> {
        self.counts.into_iter().collect()
    }
}
