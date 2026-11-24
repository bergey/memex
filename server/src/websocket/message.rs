use crate::database;
use crate::metrics;
use memex_shared::{AuthToken, Library, LibraryId, ToClient, ToServer};
use sqlx::PgPool;

use automerge::sync::SyncDoc;

#[derive(Clone, Debug)]
pub enum Reply {
  None,
  Library(LibraryId, ToClient),
  NotAuthorized,
}

pub async fn apply_message_reply(
  database: &PgPool,
  client_state: &mut super::Client,
  message: ToServer,
) -> anyhow::Result<Reply> {
  match message {
    ToServer::Authorize(token) => {
      client_state.auth_token = Some(token);
      Ok(Reply::None)
    }
    ToServer::LibraryId(id) => {
      client_state.library_in = Some(id);
      Ok(Reply::None)
    }
    ToServer::Library(am_msg) => {
      metrics::AUTOMERGE_MESSAGE_RECEIVED.inc();
      let id = client_state
        .library_in
        .ok_or_else(|| anyhow::anyhow!("sync message before Library ID set"))?;
      match authorized(database, client_state.auth_token, id).await? {
        None => Ok(Reply::NotAuthorized),
        Some(user_id) => {
          let am_state = client_state.get_am(id);
          let mut document =
            database::apply_message(&database, id, am_state, am_msg, user_id).await?;
          let _ = super::BROADCAST.send(Library {
            id,
            replicated: document.clone(),
          });
          let o_reply = document.sync().generate_sync_message(am_state);
          match o_reply {
            None => Ok(Reply::None),
            Some(msg) => Ok(Reply::Library(id, ToClient::Library(msg))),
          }
        }
      }
    }
  }
}

// keep it simple, no caching
// it's tempting to cache the expiration time, check access to a Library only on LibraryId message
// but we will want to revoke tokens in other ways, and change Library permissions
// so no caching until I'm prepared to design cache busting / broadcast changes
async fn authorized(
  database: &PgPool,
  o_auth: Option<AuthToken>,
  library_id: LibraryId,
) -> anyhow::Result<Option<i32>> {
  match o_auth {
    None => Ok(None),
    Some(auth_token) => database::authorize(&database, auth_token, library_id).await,
  }
}

#[cfg(test)]
pub mod tests {
  use super::*;
  use crate::websocket::Client;

  use std::assert_matches;

  #[sqlx::test]
  async fn set_library_id(pool: PgPool) {
    let mut client_state = Client::new();
    let l_id = LibraryId(123);
    let reply = apply_message_reply(&pool, &mut client_state, ToServer::LibraryId(l_id))
      .await
      .unwrap();
    assert_matches!(reply, Reply::None);
    assert_eq!(client_state.library_in, Some(l_id));
  }
}
