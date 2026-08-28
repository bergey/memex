use crate::database::users::user_id_internal;
use memex_shared::*;

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

pub async fn read_passkeys(
    transaction: &mut PgTransaction<'_>,
    user_id: UserId,
) -> Result<Vec<Passkey>> {
    let user_internal = user_id_internal(transaction, user_id).await?;
    let rows = query!(
        "select value from passkeys where user_id = $1",
        user_internal
    )
    .fetch_all(&mut **transaction)
    .await?;
    // TODO stream?
    let mut ret = Vec::new();
    for r in rows {
        ret.push(ciborium::from_reader::<Passkey, &[u8]>(r.value.as_ref())?)
    }
    Ok(ret)
}

// same as save_passkey_registration except for the state type
pub async fn save_passkey_authentication(
    transaction: &mut PgTransaction<'_>,
    state: PasskeyAuthentication,
    user_id: UserId,
) -> Result<()> {
    let challenge = state.challenge();
    let mut encoded = Vec::new();
    ciborium::into_writer(&state, &mut encoded)?;
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

pub async fn read_passkey_authentication(
    transaction: &mut PgTransaction<'_>,
    challenge: &[u8],
) -> Result<Option<(PasskeyAuthentication, UserId)>> {
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
            let pka = ciborium::from_reader::<PasskeyAuthentication, &[u8]>(row.state.as_ref())?;
            Ok(Some((pka, user_id)))
        }
    }
}
