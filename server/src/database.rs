use automerge::{
    AutoCommit,
    sync::{Message, SyncDoc},
};
use sqlx::*;
use uuid::Uuid;

// TODO rename before I add any more AM doc types
pub async fn apply_message(
    database: &PgPool,
    id: Uuid,
    sync_state: &mut automerge::sync::State,
    message: Message,
) -> anyhow::Result<AutoCommit> {
    let mut transaction = database.begin().await?;
    let mut document = read_library(&mut transaction, id)
        .await?
        .unwrap_or_else(|| AutoCommit::new());
    document.sync().receive_sync_message(sync_state, message)?;
    save_library(&mut transaction, id, &mut document).await?;
    transaction.commit().await?;
    Ok(document)
}

// TODO shared ID type
async fn read_library(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> anyhow::Result<Option<AutoCommit>> {
    let row: Option<Vec<u8>> = query_scalar!("select value from libraries where id = $1", id)
        .fetch_optional(&mut **transaction)
        .await?;

    Ok(row.and_then(|bytes: Vec<u8>| AutoCommit::load(bytes.as_ref()).ok()))
}

async fn save_library(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    library: &mut AutoCommit,
) -> anyhow::Result<()> {
    let bytes = library.save();
    query!(
        "insert into libraries (id, value) values ($1, $2) \
on conflict (id) do update set value = $2",
        id,
        bytes
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}
