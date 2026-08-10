use prelude::*;

use automerge::AutoCommit;
use uuid::Uuid;

// TODO shared ID type
async fn read_library(conn: sqlx::Connection, id: Uuid) -> Result<Option<AutoCommit>> {
    let row = sqlx::query!(
        "select value from libraries where id = $1",
        id
    )
        .fetch_one(&mut conn)
        .await?;

    Ok(row.map(|bytes| AutoCommit.load(&bytes)));
}
