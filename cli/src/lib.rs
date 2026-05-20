// pub mod note;
pub mod calibre;
pub mod format;
pub mod stats;
pub mod tag;
pub mod zotero;

pub struct Doc {
    pub title: String,
    pub link: String,
    pub tags: tag::Tags,
}
