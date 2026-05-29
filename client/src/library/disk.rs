use super::Library;
use crate::prelude::*;

use automerge::AutoCommit;
use indexed_db_futures::OpenDbResult;
use indexed_db_futures::database::Database;
use indexed_db_futures::prelude::*;
use indexed_db_futures::transaction::TransactionMode;
// use indexed_db_futures::typed_array::TypedArray;
use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;

impl Library {
    pub(super) fn save(&mut self) {
        let bytes = self.replicated.save();
        spawn_local(save_bytes(bytes));
    }

    pub async fn load(_id: &str) -> Library {
        Self::try_load()
            .await
            .log_error()
            .and_then(|x| {
                x.dyn_into().map_err(|_| "dyn_into failed").log_error()
            })
            .map(|array: Uint8Array| array.to_vec())
            .and_then(|bytes: Vec<u8>| AutoCommit::load(bytes.as_ref()).log_error())
            // TODO validate schema
            .map(|mut am| {
                am.update_diff_cursor();
                Self::from_replicated(am)
            })
            .unwrap_or_else(Library::new)
    }

    async fn try_load() -> OpenDbResult<JsValue> {
        let db = open_database().await?;
        let transaction = db
            .transaction("libraries")
            .with_mode(TransactionMode::Readonly)
            .build()?;
        let store = transaction.object_store("libraries")?;
        store
            .get("my_library")
            .primitive()?
            .await?
            .ok_or(js_sys::Error::new("my_library not found").into())
    }
}

async fn save_bytes(bytes: Vec<u8>) {
    try_save_bytes(bytes.as_ref()).await.log_error();
}

async fn try_save_bytes(bytes: &[u8]) -> OpenDbResult<()> {
    let db = open_database().await?;
    let transaction = db
        .transaction("libraries")
        .with_mode(TransactionMode::Readwrite)
        .build()?;
    let store = transaction.object_store("libraries")?;
    store
        .put(Into::<JsValue>::into(Uint8Array::from(bytes)))
        .with_key("my_library")
        .build()?; // TODO multiple libraries
    transaction.commit().await?;
    Ok(())
}

async fn open_database() -> OpenDbResult<Database> {
    Database::open("memex")
        .with_version(3u8)
        .with_on_blocked(|event| {
            error!("DB upgrade blocked: {:?}", event);
            Ok(())
        })
        .with_on_upgrade_needed(|event, db| {
            // Convert versions from floats to integers to allow using them in match expressions
            let old_version = event.old_version() as u64;
            let new_version = event.new_version().map(|v| v as u64);

            match (old_version, new_version) {
                (_, Some(3)) => {
                    db.create_object_store("libraries").build()?;
                }
                _ => {}
            };
            Ok(())
        })
        .await
}
