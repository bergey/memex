#[derive(PartialEq, Debug, Clone, Copy)]
pub struct TagId(u16);

pub struct Tag {
    id: TagId,
    name: String,
}

pub enum Query {
    Empty,
    Tag(TagId),
    And(&Query, &Query),
    Or(&Query, &Query),
}

// TODO benchmark BTreeSet, HashSet, persistent immutable tree

pub fn match_tags(query: &Query, orig_tags: &[TagId]) -> bool {
    let mut tags = Vec::new();
    tags.extend_from_slice(orig_tags);
    match_tags_mut(query, &mut tags)
}

fn match_tags_mut(query: &Query, tags: &mut Vec<TagId>) -> bool {
    match query {
        Query::Empty => tags.is_empty(),
        Query::Tag(id) => remove(&mut tags, id),
        Query::And(q1, q2) => match_tags_mut(q1, &mut tags) && match_tags_mut(q2, &mut tags),
        Query::Or(q1, q2) => {
            let mut tags_copy = tags.clone();
            if match_tags_mut(query, &mut tags_copy) {
                std::mem::replace(tags, tags_copy);
                true
            } else {
                let mut tags_copy = tags.clone();
                if match_tags_mut(query, &mut tags_copy) {
                    std::mem::replace(tags, tags_copy);
                    true
                } else {
                    false
                }
            }
        }
    }
}

/// return true if element was found
fn remove(tags: &mut Vec<TagId>, t: TagId) -> bool {
    for i in 0..tags.len() {
        if tags[i] == t {
            tags.swap_remove(i);
            return true;
        }
    }
    return false;
}
