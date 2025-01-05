// memex cli

use clap::Parser;
use futures_util::TryStreamExt;
use sqlx::Row;
use sqlx::sqlite::SqlitePool;
use std::collections::{HashMap, HashSet};

use memex::tag::*;

#[derive(sqlx::FromRow, Debug)]
struct ZTag {
    pub id: i64,
    pub title: String,
    pub tag: String,
}

#[derive(Eq, Hash, PartialEq)]
enum DocId {
    Zotero(i64),
    Calibre(i64),
}

struct Doc {
    // id: DocId,
    title: String,
    tags: Tags,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command()]
    query: String,
    // #[arg(long="library")]
    // libraries: Vec<String>
}

#[tokio::main()]
async fn main() -> anyhow::Result<()> {
    // let args = Cli::parse();
    let pool = SqlitePool::connect("/home/bergey/Zotero/zotero.sqlite").await?;
    let mut conn = pool.acquire().await?;

    let mut all_tags = AllTags::new();
    let mut docs: HashMap<DocId, Doc> = HashMap::new();

    // assign our own IDs, so we can merge tags from multiple libraries
    let zot_tags = {
        let mut zot_tags: HashMap<i64, TagId> = HashMap::new();
        let mut rows = sqlx::query("select tagId, name from tags").fetch(&mut *conn);
        while let Some(row) = rows.try_next().await? {
            let zid = row.try_get("tagID")?;
            let name = row.try_get("name")?;
            let memex_id = all_tags.insert(name);
            zot_tags.insert(zid, memex_id);
        }
        zot_tags
    };

    // titles
    {
        // TODO emails have subject, not title
        // exclude attachments (even if orphaned)
        let mut rows = sqlx::query("select itemID, value as title from items join itemData using (itemID) join fieldsCombined using (fieldID) join itemDataValues using (valueID) where fieldName = 'title' and itemTypeID != 3").fetch(&mut *conn);
        while let Some(row) = rows.try_next().await? {
            let zid = row.try_get("itemID")?;
            let title = row.try_get("title")?;
            docs.insert(
                DocId::Zotero(zid),
                Doc {
                    title: title,
                    tags: HashSet::new(),
                },
            );
        }
    }

    // tag the documents
    let mut rows = sqlx::query("select itemID, tagID from itemTags").fetch(&mut *conn);
    while let Some(row) = rows.try_next().await? {
        let item_id = row.try_get("itemID")?;
        let z_tag_id = row.try_get("tagID")?;
        let memex_tag_id = zot_tags
            .get(&z_tag_id)
            .ok_or(anyhow::anyhow!("tag ID violates foreign key constraint"))?;
        match docs.get_mut(&DocId::Zotero(item_id)) {
            Some(doc) => {
                doc.tags.insert(memex_tag_id.clone());
            },
            None => () //println!("skipping doc ID {item_id} has no title"),
        };
    }

    println!("{} tags", all_tags.len());
    println!("{} titles", docs.len());
    let mut tag_count = 0;
    for (_id, doc) in docs.iter() {
        tag_count += doc.tags.len();
    }
    println!("{} tag-title associations", tag_count);

    Ok(())
}
