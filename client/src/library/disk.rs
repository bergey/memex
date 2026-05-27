use super::Library;
use crate::prelude::*;

use indexed_db_futures::database::Database;
use indexed_db_futures::prelude::*;
use indexed_db_futures::transaction::TransactionMode;
use wasm_bindgen_futures::spawn_local;

impl Library {
    pub(super) fn save(&mut self) {
        let bytes = self.replicated.save();
        spawn_local(save_bytes(bytes));
    }
}

async fn save_bytes(bytes: Vec<u8>) {
    try_save_bytes(bytes).await.log_error();
}

async fn try_save_bytes(bytes: Vec<u8>) -> indexed_db_futures::OpenDbResult<()> {
    let db = Database::open("memex")
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
        .await?;

    let transaction = db
        .transaction("libraries")
        .with_mode(TransactionMode::Readwrite)
        .build()?;
    let store = transaction.object_store("libraries")?;
    store.put(bytes).with_key("my_library").build()?; // TODO multiple libraries
    transaction.commit().await?;
    Ok(())
}
