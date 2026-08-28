use crate::actor::local_actor_id;
use crate::prelude::*;
use memex_shared::library::{Library, LibraryId};

use automerge::AutoCommit;
use indexed_db_futures::OpenDbResult;
use indexed_db_futures::database::Database;
use indexed_db_futures::prelude::*;
use indexed_db_futures::transaction::TransactionMode;
use js_sys::{JsString, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;

pub fn save_library(library: &mut Library) {
    let bytes = library.replicated.save();
    spawn_local(save_bytes(library.id, bytes));
}

#[allow(dead_code)]
pub async fn load_library(id: LibraryId) -> Library {
    try_load(id)
        .await
        .log_error()
        .and_then(|jsv| decode_library(id, jsv))
        .unwrap_or_else(|| Library::new(local_actor_id()))
}

fn decode_library(id: LibraryId, jsv: JsValue) -> Option<Library> {
    jsv.dyn_into()
        .map_err(|_| "decode_library: dyn_into Uint8Array failed")
        .log_error()
        .map(|array: Uint8Array| array.to_vec())
        .and_then(|bytes: Vec<u8>| AutoCommit::load(bytes.as_ref()).log_error())
        // TODO validate schema
        .map(|mut am| {
            am.update_diff_cursor();
            Library { id, replicated: am }
        })
}

#[allow(dead_code)]
async fn try_load(id: LibraryId) -> OpenDbResult<JsValue> {
    let db = open_database().await?;
    let transaction = db
        .transaction("libraries")
        .with_mode(TransactionMode::Readonly)
        .build()?;
    let store = transaction.object_store("libraries")?;
    store
        .get(id.to_string())
        .primitive()?
        .await?
        .ok_or(js_sys::Error::new("library not found").into())
}

async fn save_bytes(id: LibraryId, bytes: Vec<u8>) {
    try_save_bytes(id, bytes.as_ref()).await.log_error();
}

async fn try_save_bytes(id: LibraryId, bytes: &[u8]) -> OpenDbResult<()> {
    let db = open_database().await?;
    let transaction = db
        .transaction("libraries")
        .with_mode(TransactionMode::Readwrite)
        .build()?;
    let store = transaction.object_store("libraries")?;
    store
        .put(Into::<JsValue>::into(Uint8Array::from(bytes)))
        .with_key(id.to_string())
        .build()?;
    transaction.commit().await?;
    Ok(())
}

pub async fn load_some_library() -> Library {
    try_load_some_library()
        .await
        .log_error()
        .flatten()
        .and_then(|(key, val)| {
            let s: String = key.dyn_into::<JsString>().ok()?.into();
            let id = LibraryId::from_str(&s)?;
            decode_library(id, val)
        })
        .unwrap_or_else(|| Library::new(local_actor_id()))
}

// Key, Value
async fn try_load_some_library() -> OpenDbResult<Option<(JsValue, JsValue)>> {
    let db = open_database().await?;
    let transaction = db
        .transaction("libraries")
        .with_mode(TransactionMode::Readonly)
        .build()?;
    let store = transaction.object_store("libraries")?;
    let mut keys = store.get_all_keys().with_limit(1).build()?.await?;
    let kv = match keys.next().transpose()? {
        Some(k) => {
            let o_value = store.get(&k).primitive()?.await?;
            o_value.map(|v| (k, v))
        }
        None => None,
    };
    Ok(kv)
}

async fn open_database() -> OpenDbResult<Database> {
    Database::open("memex")
        .with_version(3u8)
        .with_on_blocked(|event| {
            error!(?event, "DB upgrade blocked");
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
