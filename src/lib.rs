// pub mod note;
pub mod calibre;
pub mod tag;
pub mod zotero;

pub struct Doc {
    pub title: String,
    pub tags: tag::Tags,
}
