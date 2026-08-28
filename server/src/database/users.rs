use memex_shared::UserId;

use sqlx::*;

pub async fn user_id_internal(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: UserId,
) -> anyhow::Result<i32> {
    try_user_id_internal(transaction, user_id)
        .await?
        .ok_or(anyhow::anyhow!("user not found"))
}

pub async fn try_user_id_internal(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: UserId,
) -> Result<Option<i32>> {
    query_scalar!(
        "select id from users where external_id = $1",
        user_id.to_uuid()
    )
    .fetch_optional(&mut **transaction)
    .await
}

pub async fn create_user(transaction: &mut PgTransaction<'_>) -> Result<UserId> {
    let user_id = UserId::random();
    query!(
        "insert into users (external_id) values ($1)",
        user_id.to_uuid()
    )
    .execute(&mut **transaction)
    .await?;
    Ok(user_id)
}
