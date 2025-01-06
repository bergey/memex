// pub mod note;
pub mod calibre;
pub mod tag;
pub mod zotero;

#[derive(Eq, Hash, PartialEq, Debug)]
pub enum DocId {
    Zotero(i64),
    Calibre(i64),
}

pub struct Doc {
    pub title: String,
    pub tags: tag::Tags,
}
