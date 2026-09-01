use super::Library;
pub use super::ids::*;

use automerge::{self, ObjType, ReadDoc, transaction::Transactable};

// This should all be derived by macros, but write a few out to see the pattern
impl Library {
    pub fn add_record(&mut self) -> RecordId {
        let r_id = self.add_to_set(&self.records_id(), ObjType::Map);
        RecordId(r_id)
    }

    pub fn title(&self, r_id: &RecordId) -> String {
        self.get_string(&r_id.0, "title")
    }

    pub fn set_title(&mut self, r_id: &RecordId, s: &str) -> anyhow::Result<()> {
        self.replicated.put(&r_id.0, "title", s)?;
        Ok(())
    }

    pub fn set_url(&mut self, r_id: &RecordId, s: &str) -> anyhow::Result<()> {
        self.replicated.put(&r_id.0, "url", s)?;
        Ok(())
    }

    pub fn add_tag(&mut self, r_id: &RecordId, s: &str) -> anyhow::Result<TagId> {
        let tags = self.ensure_set(&r_id.0, "tags")?;
        let len = self.replicated.length(&tags);
        self.replicated.insert(&tags, len, s)?;
        let (_, tag_id) = self.replicated.get(&tags, len)?.ok_or(anyhow::anyhow!("missing tag we just inserted"))?;
        Ok(TagId(tag_id))
    }
}
