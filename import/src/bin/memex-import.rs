use memex_shared::*;

use automerge::ActorId;
use futures_util::TryStreamExt;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

#[tokio::main()]
async fn main() -> anyhow::Result<()> {
    let zots = load().await?; // load docs from Zotero

    // load AM doc from Postgres
    let mut library = Library::new(ActorId::random()); // TODO load from PG

    // import docs into AM / memex schema
    for _z in zots {
        let _r_id = library.add_record();
    }

    // save back to Postgres

    Ok(())
}

pub struct ZoteroItem {
    pub id: i32,
    pub title: String,
    pub url: String,
    pub tags: Vec<String>,
}

pub async fn load() -> anyhow::Result<Vec<ZoteroItem>> {
    let home = std::env::var("HOME")?;
    let path = format!("{home}/Zotero/zotero.sqlite");
    let pool = SqlitePool::connect(&*path).await?;
    let mut conn = pool.acquire().await?;

    // subquery for each of title, url, joining fieldData
    // subquery for creators, firstName & lastName (only one for now?)
    // let rows = sqlx::query_file_as!(ZoteroItem, "sql/zotero.sql")
    let mut rows = sqlx::query(" with titles as ( select itemID, value as title from itemData join fieldsCombined using (fieldID) join itemDataValues using (valueID) where fieldName = 'title'), urls as ( select itemID, value as url from itemData join fieldsCombined using (fieldID) join itemDataValues using (valueID) where fieldName = 'url'), tags_comma as ( select itemID, group_concat(name) as tags from itemTags join tags using (tagID) group by itemID ) select itemID, title, url, tags from titles join urls using (itemID) join tags_comma using (itemID) limit 30; ")
        .fetch(&mut *conn);
    let mut items = Vec::new();
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
