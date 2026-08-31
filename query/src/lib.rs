use std::collections::HashSet;
use std::hash::Hash;
use winnow::combinator::{alt, delimited, opt, separated, terminated};
use winnow::prelude::*;
use winnow::token::*;
use winnow::error::Result;

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
    Not(Box<Query<T>>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operator {
    And,
    Or,
}

pub fn match_tags<T>(query: &Query<T>, item_tags: &HashSet<T>) -> bool
where
    T: Eq + Hash,
{
    match query {
        Query::Tag(t) => item_tags.contains(t),
        Query::Only(ts) => item_tags.is_subset(ts),
        Query::Function(Operator::And, args) => args.iter().all(|q| match_tags(q, item_tags)),
        Query::Function(Operator::Or, args) => args.iter().any(|q| match_tags(q, item_tags)),
        Query::Not(q) => !match_tags(q, item_tags),
    }
}

const QUOTATION_MARKS: [char; 2] = ['\'', '"'];
const SPACE: [char; 2] = [' ', '\t'];
const NOT_TAG: [char; 4] = [' ', '\t', '(', ')'];

type ResultQ = Result<Query<String>>;

/// single or double quote
fn quoted_tag(input: &mut &str) -> Result<String> {
    let quote = one_of(QUOTATION_MARKS).parse_next(input)?;
    let tag = take_until(1.., quote).parse_next(input)?;
    Ok(tag.to_string())
}

fn bare_tag(input: &mut &str) -> Result<String> {
    let tag = take_till(1.., NOT_TAG).parse_next(input)?;
    Ok(tag.to_string())
}

fn tag(input: &mut &str) -> Result<String> {
    alt((quoted_tag, bare_tag)).parse_next(input)
}

fn query_tag(input: &mut &str) -> ResultQ {
    let t = tag.parse_next(input)?;
    Ok(Query::Tag(t))
}

fn space(input: &mut &str) -> Result<()> {
    let _ = take_while(1.., SPACE).parse_next(input)?;
    Ok(())
}

/// within parens, 1 or more tags separated by whitespace
fn only(input: &mut &str) -> ResultQ {
    let tags: Vec<String> = delimited(
        ("(only", space),
        separated(1.., tag, space),
        (opt(space), ')'),
    )
        .parse_next(input)?;
    let mut set = HashSet::new();
    for t in tags {
        set.insert(t);
    }
    Ok(Query::Only(set))
}

fn op(input: &mut &str) -> Result<Operator> {
    use Operator::*;
    alt(("and".value(And), "or".value(Or))).parse_next(input)
}

fn function(input: &mut &str) -> ResultQ {
    let (op, args) = delimited(
        '(',
        (terminated(op, space), separated(1.., query, space)),
        (opt(space), ')'),
    )
    .parse_next(input)?;
    Ok(Query::Function(op, args))
}

fn not(input: &mut &str) -> ResultQ {
    let arg = delimited(("(not", space), query, (opt(space), ')')).parse_next(input)?;
    Ok(Query::Not(Box::new(arg)))
}

fn query(input: &mut &str) -> ResultQ {
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
