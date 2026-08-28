mod database;

use crate::database::save_auth_token;
use crate::database::users::*;
use crate::prelude::*;
use memex_shared::*;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
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
    database::save_passkey_registration(&mut transaction, server_secret, user_id).await?;
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
    let (reg, user_id) = database::read_passkey_registration(&mut transaction, &challenge)
        .await?
        .ok_or(HttpError {
            error: anyhow::anyhow!("registration not found or expired"),
            status_code: StatusCode::CONFLICT,
        })?;
    let passkey = WEBAUTHN.finish_passkey_registration(&body.0, &reg)?;
    database::save_passkey(&mut transaction, &passkey, user_id).await?; // delete registration
    let auth_token = AuthToken::random();
    save_auth_token(&mut transaction, auth_token, user_id).await?;
    transaction.commit().await?;
    Ok(Json(auth_token))
}

/// login on a device that has a passkey
pub async fn login_start(
    State(pools): Pools,
    Path(user_id): Path<UserId>,
) -> HttpResult<Json<RequestChallengeResponse>> {
    let mut transaction = pools.postgres.begin().await?;
    let passkeys = database::read_passkeys(&mut transaction, user_id).await?;
    let (to_client, server_secret) = WEBAUTHN.start_passkey_authentication(&passkeys)?;
    database::save_passkey_authentication(&mut transaction, server_secret, user_id).await?;
    transaction.commit().await?;
    Ok(Json(to_client))
}

pub async fn login_finish(
    State(db): Pools,
    body: Json<PublicKeyCredential>,
) -> HttpResult<Json<AuthToken>> {
    let attestation: AuthenticatorAssertionResponse<webauthn_rs_core::Authentication> =
        AuthenticatorAssertionResponse::try_from(&body.0.response)?;
    let challenge = attestation.challenge();
    let mut transaction = db.postgres.begin().await?;

    let (pka, user_id) = database::read_passkey_authentication(&mut transaction, &challenge)
        .await?
        .ok_or(HttpError {
            error: anyhow::anyhow!("registration not found or expired"),
            status_code: StatusCode::CONFLICT,
        })?;
    let _auth_info = WEBAUTHN.finish_passkey_authentication(&body.0, &pka)?;
    // TODO check signature counter?

    let auth_token = AuthToken::random();
    save_auth_token(&mut transaction, auth_token, user_id).await?;
    transaction.commit().await?;
    Ok(Json(auth_token))
}
