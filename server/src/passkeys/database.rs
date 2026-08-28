use memex_shared::*;
use crate::database::users::user_id_internal;

use anyhow::Result;
use sqlx::*;
use webauthn_rs::prelude::*;

pub async fn save_passkey_registration(
    transaction: &mut PgTransaction<'_>,
    reg: PasskeyRegistration,
    user_id: UserId,
) -> Result<()> {
    let challenge = reg.challenge();
    let mut encoded = Vec::new();
    ciborium::into_writer(&reg, &mut encoded)?;
    let user_internal = user_id_internal(transaction, user_id).await?;
    let _ = query!(
        "insert into passkey_challenges (challenge, user_id, state) values ($1, $2, $3)",
        challenge,
        user_internal,
        encoded
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

pub async fn save_passkey(
    transaction: &mut PgTransaction<'_>,
    passkey: &Passkey,
    user_id: UserId,
) -> Result<()> {
    let mut encoded = Vec::new();
    ciborium::into_writer(passkey, &mut encoded)?;
    query!(
        "insert into passkeys (cred_id, user_id, value) values ($1, $2, $3)",
        passkey.cred_id().as_ref(),
        user_id_internal(transaction, user_id).await?,
        encoded
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

pub async fn read_passkey_registration(
    transaction: &mut PgTransaction<'_>,
    challenge: &[u8],
) -> Result<Option<(PasskeyRegistration, UserId)>> {
    let o_row = query!(
        "select users.external_id, state \
        from passkey_challenges join users on users.id = user_id \
        where challenge = $1 and expires > now()",
        challenge
    )
    .fetch_optional(&mut **transaction)
    .await?;
    match o_row {
        None => Ok(None),
        Some(row) => {
            let user_id = UserId::from_uuid(&row.external_id);
            let reg = ciborium::from_reader::<PasskeyRegistration, &[u8]>(row.state.as_ref())?;
            Ok(Some((reg, user_id)))
        }
    }
}
