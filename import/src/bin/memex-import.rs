use memex_shared::*;

use clap::Parser;
use automerge::{ActorId, AutoCommit};
use futures_util::TryStreamExt;
use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::SqlitePool;
use sqlx::*;
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short, default_value_t = 0)]
    library_id: u128,
    #[arg(long)]
    limit: Option<i32>,
    #[arg(long, short, default_value="postgres://memex:memex@localhost:5432/memex-dev")]
    database: String,
}

#[tokio::main()]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let postgres = PgPoolOptions::new()
        .max_connections(1)
        .connect(&args.database)
        .await?;

    // load AM doc from Postgres
    let library_id = LibraryId(args.library_id);
    let actor_id = ActorId::random();
    let mut transaction = postgres.begin().await?;
    let mut library = {
        let o_document = read_postgres(&mut transaction, library_id).await?;
        match o_document {
            Some(mut document) => {
                document.set_actor(actor_id);
                Library {
                    id: library_id,
                    replicated: document,
                }
            }
            None => Library::new(actor_id),
        }
    };

    // import docs into AM / memex schema
    let zots = load_zotero(args.limit).await?; // load docs from Zotero
    for (i, z) in zots.iter().enumerate() {
        let r_id = library.add_record();
        library.set_title(&r_id, &z.title)?;
        library.set_url(&r_id, &z.url)?;
        for t in &z.tags {
            library.add_tag(&r_id, t)?;
        }
        if i % 100 == 0 {
            tracing::info!("{i} records");
        }
    }

    save_postgres(&mut transaction, library_id, &mut library.replicated).await?;
    transaction.commit().await?;
    tracing::info!("imported {} records", zots.len());

    Ok(())
}

pub struct ZoteroItem {
    pub id: i32,
    pub title: String,
    pub url: String,
    pub tags: Vec<String>,
}

pub async fn load_zotero(limit: Option<i32>) -> anyhow::Result<Vec<ZoteroItem>> {
    let home = std::env::var("HOME")?;
    let path = format!("{home}/Zotero/zotero.sqlite");
    let pool = SqlitePool::connect(&*path).await?;
    let mut conn = pool.acquire().await?;

    // subquery for each of title, url, joining fieldData
    // subquery for creators, firstName & lastName (only one for now?)
    // let rows = sqlx::query_file_as!(ZoteroItem, "sql/zotero.sql")
    let mut rows = sqlx::query(" with titles as ( select itemID, value as title from itemData join fieldsCombined using (fieldID) join itemDataValues using (valueID) where fieldName = 'title'), urls as ( select itemID, value as url from itemData join fieldsCombined using (fieldID) join itemDataValues using (valueID) where fieldName = 'url'), tags_comma as ( select itemID, group_concat(name) as tags from itemTags join tags using (tagID) group by itemID ) select itemID, title, url, tags from titles join urls using (itemID) join tags_comma using (itemID) limit $1").bind(limit.unwrap_or(100_000))
        .fetch(&mut *conn);
    let mut items = Vec::new();
    // might be nicer to expose the stream, materialize only one ZoteroItem at a time
    // Could likely borrow &str in that case, maybe expose an iterator over tags also
    while let Some(row) = rows.try_next().await? {
        let id = row.try_get("itemID")?;
        let title = row.try_get("title")?;
        let url = row.try_get("url")?;
        let tags_comma: &str = row.try_get("tags")?;
        items.push(ZoteroItem {
            tags: tags_comma.split(',').map(|s| s.to_string()).collect(),
            id,
            title,
            url,
        })
    }
    Ok(items)
}

// cf server::database

async fn read_postgres(
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

async fn save_postgres(
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
