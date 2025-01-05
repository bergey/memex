use super::{TagId, AllTags};
use std::collections::HashSet;
use winnow::prelude::*;
use winnow::token::*;

/// simplest query: rust
/// all features:
/// (safety | memory-safety | 'memory safety') & only(safety, memory-safety, 'memory safety')
pub enum Query<T> {
    Only(HashSet<T>),
    Tag(T),
    /// matches single tag
    And(Box<Query<T>>, Box<Query<T>>),
    Or(Box<Query<T>>, Box<Query<T>>),
}

impl Query<String> {
    pub fn compile(self, tags: &mut AllTags) -> Query<TagId> {
        use Query::*;
        match self {
            Tag(s) => {
                let id = tags.insert(s);
                Tag(id)
            },
            Only(ts) => {
                let mut ids = HashSet::new();
                for t in ts {
                    ids.insert(tags.insert(t));
                }
                Only(ids)
            },
            And(l, r) => And(Box::new(l.compile(tags)), Box::new(r.compile(tags))),
            Or(l, r) => Or(Box::new(l.compile(tags)), Box::new(r.compile(tags))),
        }
    }
}

const QUOTATION_MARKS: [char; 2] = ['\'', '"'];
const SPACE: [char; 2] = [' ', '\t'];

/// single or double quote
fn quoted_tag<'s>(input: &mut &'s str) -> PResult<Query<String>> {
    let quote = one_of(QUOTATION_MARKS).parse_next(input)?;
    let tag = take_until(1.., quote).parse_next(input)?;
    Ok(Query::Tag(tag.to_string()))
}

fn bare_tag<'s>(input: &mut &'s str) -> PResult<Query<String>> {
    let tag = take_till(1.., SPACE).parse_next(input)?;
    Ok(Query::Tag(tag.to_string()))
}
