pub mod users;

use memex_shared::*;

use automerge::{
  AutoCommit,
  sync::{Message, SyncDoc},
};
use memex_shared::{AuthToken, LibraryId};
use sqlx::*;

// TODO rename before I add any more AM doc types
pub async fn apply_message(
  database: &PgPool,
  id: LibraryId,
  sync_state: &mut automerge::sync::State,
  message: Message,
  owner: i32,
) -> anyhow::Result<AutoCommit> {
  let mut transaction = database.begin().await?;
  let mut document = read_library(&mut transaction, id)
    .await?
    .unwrap_or_else(|| AutoCommit::new());
  document.sync().receive_sync_message(sync_state, message)?;
  save_library(&mut transaction, id, &mut document, owner).await?;
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
      .await?
      .flatten(); // TODO null value should be eq new empty Library?

  Ok(row.and_then(|bytes: Vec<u8>| AutoCommit::load(bytes.as_ref()).ok()))
}

async fn save_library(
  transaction: &mut Transaction<'_, Postgres>,
  id: LibraryId,
  library: &mut AutoCommit,
  owner: i32,
) -> anyhow::Result<()> {
  let bytes = library.save();
  query!(
    "insert into libraries (id, value, owner) values ($1, $2, $3) \
on conflict (id) do update set value = $2",
    id.to_uuid(),
    bytes,
    owner
  )
  .execute(&mut **transaction)
  .await?;
  Ok(())
}

pub async fn authorize(
  database: &PgPool,
  auth_token: AuthToken,
  library_id: LibraryId,
) -> anyhow::Result<Option<i32>> {
  // user.id not external ID
  let mut transaction = database.begin().await?;
  let owner = query_scalar!(
    "select owner from libraries where id = $1",
    library_id.to_uuid()
  )
  .fetch_optional(&mut *transaction)
  .await?;
  let user = query_scalar!(
    "select user_id from auth_tokens where auth_tokens.id = $1 and expires > now()",
    auth_token.to_uuid()
  )
  .fetch_optional(&mut *transaction)
  .await?;
  transaction.commit().await?;
  match (owner, user) {
    (None, Some(Some(user))) => Ok(Some(user)), // any user may create a new library
    (Some(Some(owner)), Some(Some(user))) if owner == user => Ok(Some(owner)),
    _ => Ok(None), // either invalid token or user does not own this library
  }
}

pub async fn save_auth_token(
  transaction: &mut PgTransaction<'_>,
  auth_token: AuthToken,
  user_id: UserId,
) -> anyhow::Result<()> {
  let user_internal = users::user_id_internal(transaction, user_id).await?;
  query!(
    "insert into auth_tokens (id, user_id, expires) values ($1, $2, now() + interval '5m')",
    auth_token.to_uuid(),
    user_internal
  )
  .execute(&mut **transaction)
  .await?;
  Ok(())
}

#[cfg(test)]
pub mod tests {
  use super::*;
  use sqlx::PgPool;

  #[sqlx::test]
  async fn authorize_empty(pool: PgPool) -> anyhow::Result<()> {
    let o_user_row_id = authorize(&pool, AuthToken(0), LibraryId::random()).await?;
    assert_eq!(o_user_row_id, None);
    Ok(())
  }
}
