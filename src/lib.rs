// pub mod note;
pub mod calibre;
pub mod tag;
pub mod zotero;
pub mod stats;

pub struct Doc {
    pub title: String,
    pub link: String,
    pub tags: tag::Tags,
}
