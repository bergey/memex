use super::{AllTags, TagId};
use std::collections::HashSet;
use std::hash::Hash;
use winnow::combinator::{alt, delimited, separated};
use winnow::error::{ContextError, ErrMode};
use winnow::prelude::*;
use winnow::token::*;

/// simplest query: rust
/// all features:
/// (safety | memory-safety | 'memory safety') & only(safety, memory-safety, 'memory safety')
#[derive(Debug, PartialEq)]
pub enum Query<T: Eq + Hash> {
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
            }
            Only(ts) => {
                let mut ids = HashSet::new();
                for t in ts {
                    ids.insert(tags.insert(t));
                }
                Only(ids)
            }
            And(l, r) => And(Box::new(l.compile(tags)), Box::new(r.compile(tags))),
            Or(l, r) => Or(Box::new(l.compile(tags)), Box::new(r.compile(tags))),
        }
    }
}

const QUOTATION_MARKS: [char; 2] = ['\'', '"'];
const SPACE: [char; 2] = [' ', '\t'];
const NOT_TAG: [char; 4] = [' ', '\t', '(', ')'];

type Result = PResult<Query<String>>;

/// single or double quote
fn quoted_tag<'s>(input: &mut &'s str) -> PResult<String> {
    let quote = one_of(QUOTATION_MARKS).parse_next(input)?;
    let tag = take_until(1.., quote).parse_next(input)?;
    Ok(tag.to_string())
}

fn bare_tag<'s>(input: &mut &'s str) -> PResult<String> {
    let tag = take_till(1.., NOT_TAG).parse_next(input)?;
    Ok(tag.to_string())
}

fn tag<'s>(input: &mut &'s str) -> PResult<String> {
    alt((quoted_tag, bare_tag)).parse_next(input)
}

fn query_tag<'s>(input: &mut &'s str) -> Result {
    let t = tag.parse_next(input)?;
    Ok(Query::Tag(t))
}

/// within parens, 1 or more tags separated by a comma & optional whitespace
fn only<'s>(input: &mut &'s str) -> Result {
    let tags: Vec<String> = delimited(
        "only(",
        separated(1.., tag, (',', take_while(0.., SPACE))),
        ')',
    )
    .parse_next(input)?;
    let mut set = HashSet::new();
    for t in tags {
        set.insert(t);
    }
    Ok(Query::Only(set))
}

fn binop<'s>(input: &mut &'s str) -> Result {
    let (l, op, r) = (query, one_of(['&', '|']), query).parse_next(input)?;
    match op {
        '&' => Ok(Query::And(Box::new(l), Box::new(r))),
        '|' => Ok(Query::Or(Box::new(l), Box::new(r))),
        _ => Err(ErrMode::Cut(ContextError::new())),
    }
}

fn space(&mut str) -> Result {
    take_while(0.., SPACE).parse_next(input)
}

fn parens(&mut str) -> Result {
    delimited(('(', space), query, (space, ')')).parse_next(input)
}

fn query<'s>(input: &mut &'s str) -> Result {
    // alt((binop, only, query_tag)).parse_next(input)
    separated
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use Query::*;

    #[test]
    fn foo() {
        assert_eq!(query(&mut "foo"), Ok(Tag("foo".to_string())))
    }

    #[test]
    fn only_foo() {
        assert_eq!(
            query(&mut "only(foo)"),
            Ok(Only(HashSet::from(["foo".to_string()])))
        )
    }

    #[test]
    fn only_foo_bar() {
        assert_eq!(
            query(&mut "only(foo, bar))"),
            Ok(Only(HashSet::from(["foo".to_string(), "bar".to_string()])))
        )
    }

    #[test]
    fn and() {
        assert_eq!(
            query(&mut "foo & bar"),
            Ok(And(
                Box::new(Tag("foo".to_string())),
                Box::new(Tag("bar".to_string()))
            ))
        )
    }

    #[test]
    fn or() {
        assert_eq!(
            query(&mut "foo | bar"),
            Ok(Or(
                Box::new(Tag("foo".to_string())),
                Box::new(Tag("bar".to_string()))
            ))
        )
    }
}
