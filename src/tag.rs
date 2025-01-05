use std::collections::HashSet;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct TagId(u16);

pub struct Tag {
    id: TagId,
    name: String,
}

pub type Tags = HashSet<TagId>;

pub enum Query<'a> {
    Only(Tags),
    Tag(TagId),
    And(&'a Query<'a>, &'a Query<'a>),
    Or(&'a Query<'a>, &'a Query<'a>),
}

pub fn match_tags(query: &Query, item_tags: &Tags) -> bool {
    match query {
        Query::Tag(t) => item_tags.contains(t),
        Query::And(l, r) => match_tags(l, item_tags) && match_tags(r, item_tags),
        Query::Or(l, r) => match_tags(l, item_tags) || match_tags(r, item_tags),
        Query::Only(ts) => !item_tags.is_subset(ts),
    }
}
