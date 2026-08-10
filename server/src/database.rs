use automerge::AutoCommit;
use sqlx::*;
use uuid::Uuid;

// TODO shared ID type
pub async fn read_library(transaction: &mut Transaction<'_, Postgres>, id: Uuid) -> anyhow::Result<Option<AutoCommit>> {
    let row: Option<Vec<u8>> = query_scalar!("select value from libraries where id = $1", id)
        .fetch_optional(&mut **transaction)
        .await?;

    Ok(row.and_then(|bytes: Vec<u8>| AutoCommit::load(bytes.as_ref()).ok()))
}


pub async fn save_library(transaction: &mut Transaction<'_, Postgres>, id: Uuid, library: &mut AutoCommit) -> anyhow::Result<()> {
    let bytes = library.save();
    query!("insert into libraries (id, value) values ($1, $2) \
on conflict (id) do update set value = $2", id, bytes)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}
