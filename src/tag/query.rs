use super::{AllTags, TagId};
use std::collections::HashSet;
use std::hash::Hash;
use winnow::combinator::{alt, delimited, separated, terminated, opt};
use winnow::prelude::*;
use winnow::token::*;

/// simplest query: rust
/// all features:
/// (and (or safety memory-safety 'memory safety') (only safety memory-safety 'memory safety'))
#[derive(Debug, PartialEq)]
pub enum Query<T: Eq + Hash> {
    Only(HashSet<T>),
    /// matches single tag
    Tag(T),
    Function(Operator, Vec<Query<T>>),
    /// unary
    Not(Box<Query<T>>)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operator {
    And,
    Or,
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
            Function(op, args) => Function(op, args.into_iter().map(|q| q.compile(tags)).collect()),
            Not(q) => Not(Box::new(q.compile(tags))),
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

fn space(input: &mut &str) -> PResult<()> {
    let _ = take_while(1.., SPACE).parse_next(input)?;
    Ok(())
}

/// within parens, 1 or more tags separated by a comma & optional whitespace
fn only<'s>(input: &mut &'s str) -> Result {
    let tags: Vec<String> =
        delimited(terminated("(only", space), separated(1.., tag, space), ')').parse_next(input)?;
    let mut set = HashSet::new();
    for t in tags {
        set.insert(t);
    }
    Ok(Query::Only(set))
}

fn op(input: &mut &str) -> PResult<Operator> {
    use Operator::*;
    alt(("and".value(And), "or".value(Or))).parse_next(input)
}

fn function<'s>(input: &mut &'s str) -> Result {
    let (op, args) = delimited(
        '(',
        (terminated(op, space), separated(1.., query, space)),
        ')',
    )
    .parse_next(input)?;
    Ok(Query::Function(op, args))
}

fn not(input: &mut &str) -> Result {
    let arg = delimited(
        ("(not", space),
        query,
        (opt(space), ')')
    ).parse_next(input)?;
    Ok(Query::Not(Box::new(arg)))
}

fn query<'s>(input: &mut &'s str) -> Result {
    alt((only, function, not, query_tag)).parse_next(input)
}

pub fn parse_query(input: &str) -> anyhow::Result<Query<String>> {
    query.parse(input).map_err(|e| anyhow::format_err!("{e}"))
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
            query(&mut "(only foo)"),
            Ok(Only(HashSet::from(["foo".to_string()])))
        )
    }

    #[test]
    fn only_foo_bar() {
        assert_eq!(
            query(&mut "(only foo bar))"),
            Ok(Only(HashSet::from(["foo".to_string(), "bar".to_string()])))
        )
    }

    #[test]
    fn op_and() {
        assert_eq!(op(&mut "and"), Ok(Operator::And))
    }

    #[test]
    fn and() {
        assert_eq!(
            query(&mut "(and foo bar)"),
            Ok(Function(
                Operator::And,
                Vec::from([Tag("foo".to_string()), Tag("bar".to_string())])
            ))
        )
    }

    #[test]
    fn or() {
        assert_eq!(
            query(&mut "(or foo bar)"),
            Ok(Function(
                Operator::Or,
                Vec::from([Tag("foo".to_string()), Tag("bar".to_string())])
            ))
        )
    }

    #[test]
    fn not() {
        assert_eq!(
            query(&mut "(not foo)"),
            Ok(Not(Box::new(Tag("foo".to_string()))))
        )
    }
}
