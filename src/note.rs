use crate::tag::TagId;

use std::time::SystemTime;

pub struct Note {
    title: String,
    body: String,
    created_at: SystemTime,
    tags: Vec<TagId>,
}
