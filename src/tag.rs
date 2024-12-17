#[derive(PartialEq, Debug)]
pub struct TagId(u16);

pub struct Tag {
    id: TagId,
    name: String,
}

pub enum Query {
    Tag(TagId),
}

pub fn match_tags(query: Query, tags: Vec<TagId>) -> bool {
    match query {
        Query::Tag(id) => tags.contains(&id)
    }
}
    
