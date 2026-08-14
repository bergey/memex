// helpers for CRDT access, defaults
// not specific to this application / schema

use super::Library;
use crate::errors::LogResult;

use automerge::{self, ObjId, ObjType, ReadDoc, error::AutomergeError, transaction::Transactable};

type R<T> = Result<T, AutomergeError>;

impl Library {
    pub(super) fn ensure_set(&mut self, id: &ObjId, key: &str) -> R<ObjId> {
        let id = match self.replicated.get(id, key)? {
            Some((_, id)) => id,
            None => self.replicated.put_object(id, key, ObjType::List)?,
        };
        Ok(id)
    }

    pub(super) fn add_to_set(&mut self, set_id: &ObjId, obj_type: ObjType) -> ObjId {
        let len = self.replicated.length(set_id);
        self.replicated
            .insert_object(set_id, len, obj_type)
            .unwrap()
    }

    pub(super) fn remove_from_set(&mut self, set_id: &ObjId, item_id: &ObjId) {
        let o_index = self
            .replicated
            .values(set_id)
            .position(|(_, id)| id == *item_id);
        if let Some(index) = o_index {
            self.replicated
                .splice(set_id, index, 1, std::iter::empty::<&str>())
                .unwrap();
        }
    }

    // empty set if nothing is present at the given key
    pub(super) fn iter_set(
        &self,
        set_id: &ObjId,
        key: &str,
    ) -> Result<automerge::iter::Values<'_>, AutomergeError> {
        let o_id = self.replicated.get(&set_id, key)?;
        let iter = match o_id {
            None => Default::default(),
            Some((_, id)) => self.replicated.values(id),
        };
        Ok(iter)
    }

    pub(super) fn get_string(&self, r_id: &ObjId, field: &str) -> String {
        self.replicated
            .get(&r_id, field)
            .log_error()
            .flatten()
            .map(|a| a.0.into_string().ok())
            .flatten()
            .unwrap_or_else(|| "".to_string())
    }

    pub(super) fn get_i64(&self, r_id: &ObjId, field: &str) -> Option<i64> {
        self.replicated
            .get(&r_id, field)
            .log_error()
            .flatten()
            .and_then(|a| a.0.to_i64())
    }
}
