pub mod query;
use query::Query;

use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct TagId(pub usize);

pub type Tags = HashSet<TagId>;

pub struct AllTags {
    names: Vec<String>,
    ids: HashMap<String, TagId>
}

impl AllTags {
    pub fn new() -> Self {
        AllTags {
            names: Vec::new(),
            ids: HashMap::new(),
        }
    }

    pub fn name(&self, id: TagId) -> Option<String> {
        self.names.get(id.0).cloned()
    }

    pub fn id(&self, name: &str) -> Option<TagId> {
        self.ids.get(name).copied()
    }

    // returns a new TagId or the ID of an existing tag
    pub fn insert(&mut self, name: String) -> TagId {
        match self.id(&name) {
            Some(id) => id,
            None => {
                let id = TagId(self.names.len());
                self.names.push(name.clone());
                self.ids.insert(name, id);
                id
            }
        }
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }
}

pub fn match_tags(query: &Query<TagId>, item_tags: &Tags) -> bool {
    match query {
        Query::Tag(t) => item_tags.contains(t),
        Query::And(l, r) => match_tags(l, item_tags) && match_tags(r, item_tags),
        Query::Or(l, r) => match_tags(l, item_tags) || match_tags(r, item_tags),
        Query::Only(ts) => item_tags.is_subset(ts),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn empty() -> Tags {
        HashSet::from([])
    }
    fn a() -> Tags {
        HashSet::from([TagId(1)])
    }
    fn ab() -> Tags {
        HashSet::from([TagId(1), TagId(2)])
    }
    fn abc() -> Tags {
        HashSet::from([TagId(1), TagId(2), TagId(3)])
    }

    #[test]
    fn only_a_empty() {
        assert!(match_tags(&Query::Only(a()), &empty()))
    }

    fn assert_only(query_tags: Tags, item_tags: Tags, truth: bool) {
        assert_eq!(match_tags(&Query::Only(query_tags), &item_tags), truth);
    }

    #[test]
    fn only_a_a() {
        assert_only(a(), a(), true);
    }

    #[test]
    fn only_a_ab() {
        assert_only(a(), ab(), false);
    }
    #[test]
    fn only_ab_empty() {
        assert_only(ab(), empty(), true);
    }
    #[test]
    fn only_ab_a() {
        assert_only(ab(), a(), true);
    }
    #[test]
    fn only_ab_ab() {
        assert_only(ab(), ab(), true);
    }
    #[test]
    fn only_ab_abc() {
        assert_only(ab(), abc(), false);
    }

    fn assert_a_only_a(item_tags: Tags, truth: bool) {
        let tagged = Query::Tag(TagId(1));
        let only = Query::Only(a());
        assert_eq!(match_tags(&Query::And(&tagged, &only), &item_tags), truth);
    }

    #[test]
    fn a_only_a_empty() {
        assert_a_only_a(empty(), false);
    }

    #[test]
    fn a_only_a_a() {
        assert_a_only_a(a(), true);
    }

    #[test]
    fn a_only_a_ab() {
        assert_a_only_a(ab(), false);
    }
}
