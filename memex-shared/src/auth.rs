use crate::library::ids::{AuthToken, UserId};

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Clone, Debug, Deserialize, Serialize)]
pub struct AuthResponse {
  pub user_id: UserId,
  pub auth_token: AuthToken,
  // refresh token
}
