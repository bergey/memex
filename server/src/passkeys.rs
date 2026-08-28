use crate::database::*;
use crate::prelude::*;
use memex_shared::*;

use anyhow::Result;
use axum::{Json, extract::State, http::StatusCode};
use sqlx::*;
use webauthn_rs::prelude::*;

lazy_static! {
    // TODO env vars
    static ref WEBAUTHN: Webauthn = {
        let rp_id = "teallabs.org";
        let rp_origin = Url::parse("https://teallabs.org").expect("Invalid URL");
        let builder = WebauthnBuilder::new(rp_id, &rp_origin).expect("Invalid configuration");
        builder.build().expect("Invalid configuration")
    };
}

/// new User, no prior credentials
pub async fn signup_start(State(pools): Pools) -> HttpResult<Json<CreationChallengeResponse>> {
    let mut transaction = pools.postgres.begin().await?;
    let user_id = create_user(&mut transaction).await?;
    let (to_client, server_secret) =
        WEBAUTHN.start_passkey_registration(user_id.to_uuid(), "", "", None)?;
    save_passkey_registration(&mut transaction, server_secret, user_id).await?;
    transaction.commit().await?;
    Ok(Json(to_client))
}

pub async fn signup_finish(
    State(db): Pools,
    body: Json<RegisterPublicKeyCredential>,
) -> HttpResult<Json<AuthToken>> {
    let attestation: AuthenticatorAttestationResponse<webauthn_rs_core::Registration> =
        AuthenticatorAttestationResponse::try_from(&body.0.response)?;
    let challenge = attestation.challenge();
    let mut transaction = db.postgres.begin().await?;
    let (reg, user_id) = read_passkey_registration(&mut transaction, &challenge)
        .await?
        .ok_or(HttpError {
            error: anyhow::anyhow!("registration not found or expired"),
            status_code: StatusCode::CONFLICT,
        })?;
    let passkey = WEBAUTHN.finish_passkey_registration(&body.0, &reg)?;
    save_passkey(&mut transaction, &passkey, user_id).await?; // delete registration
    let auth_token = AuthToken::random();
    // save_auth_token(&mut transaction, auth_token).await?;
    transaction.commit().await?;
    Ok(Json(auth_token))
}

async fn save_passkey_registration(
    transaction: &mut Transaction<'_, Postgres>,
    reg: PasskeyRegistration,
    user_id: UserId,
) -> Result<()> {
    let challenge = reg.challenge();
    let mut encoded = Vec::new();
    ciborium::into_writer(&reg, &mut encoded)?;
    let user_id_internal = query_scalar!(
        "select id from users where external_id = $1",
        user_id.to_uuid()
    )
    .fetch_optional(&mut **transaction)
    .await?;
    let _ = query!(
        "insert into passkey_challenges (challenge, user_id, state) values ($1, $2, $3)",
        challenge,
        user_id_internal,
        encoded
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

async fn save_passkey(
    transaction: &mut Transaction<'_, Postgres>,
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

async fn create_user(transaction: &mut Transaction<'_, Postgres>) -> Result<UserId> {
    let user_id = UserId::random();
    query!(
        "insert into users (external_id) values ($1)",
        user_id.to_uuid()
    )
    .execute(&mut **transaction)
    .await?;
    Ok(user_id)
}

async fn read_passkey_registration(
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
