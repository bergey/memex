// pub mod note;
pub mod tag;

use std::collections::HashSet;

pub struct Doc {
    pub title: String,
    pub tags: tag::Tags,
}

pub fn fake_documents() -> Vec<Doc> {
    use tag::TagId;

    let mut ret = Vec::new();

    let tags = HashSet::from([TagId(1)]);
    ret.push(Doc { title: "A".to_string(), tags });

    let tags = HashSet::from([TagId(1), TagId(2)]);
    ret.push(Doc { title: "B".to_string(), tags });

    let tags = HashSet::from([TagId(1), TagId(2), TagId(3)]);
    ret.push(Doc { title: "C".to_string(), tags });

    let tags = HashSet::from([]);
    ret.push(Doc { title: "D".to_string(), tags });

    ret
}

