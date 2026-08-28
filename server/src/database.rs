pub mod users;

use memex_shared::*;

use automerge::{
    AutoCommit,
    sync::{Message, SyncDoc},
};
use memex_shared::{AuthToken, LibraryId};
use sqlx::types::time::OffsetDateTime;
use sqlx::*;

// TODO rename before I add any more AM doc types
pub async fn apply_message(
    database: &PgPool,
    id: LibraryId,
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

async fn read_library(
    transaction: &mut Transaction<'_, Postgres>,
    id: LibraryId,
) -> anyhow::Result<Option<AutoCommit>> {
    let row: Option<Vec<u8>> =
        query_scalar!("select value from libraries where id = $1", id.to_uuid())
            .fetch_optional(&mut **transaction)
            .await?;

    Ok(row.and_then(|bytes: Vec<u8>| AutoCommit::load(bytes.as_ref()).ok()))
}

async fn save_library(
    transaction: &mut Transaction<'_, Postgres>,
    id: LibraryId,
    library: &mut AutoCommit,
) -> anyhow::Result<()> {
    let bytes = library.save();
    query!(
        "insert into libraries (id, value) values ($1, $2) \
on conflict (id) do update set value = $2",
        id.to_uuid(),
        bytes
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

pub async fn authorize(
    database: &PgPool,
    auth_token: AuthToken,
    library_id: LibraryId,
) -> anyhow::Result<bool> {
    let mut transaction = database.begin().await?;
    let expires = query_scalar!("select expires from auth_tokens join libraries on user_id = owner where auth_tokens.id = $1 and libraries.id = $2", auth_token.to_uuid(), library_id.to_uuid())
        .fetch_optional(&mut *transaction)
        .await?;
    transaction.commit().await?;
    match expires {
        None => Ok(false),
        Some(expires) => Ok(OffsetDateTime::now_utc() < expires),
    }
}

pub async fn save_auth_token(
    transaction: &mut PgTransaction<'_>,
    auth_token: AuthToken,
    user_id: UserId,
) -> anyhow::Result<()> {
    let user_internal = users::user_id_internal(transaction, user_id)
        .await?
        .ok_or(anyhow::anyhow!("user not found"))?;
    query!(
        "insert into auth_tokens (id, user_id, expires) values ($1, $2, now() + interval '5m')",
        auth_token.to_uuid(),
        user_internal
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}
