use super::Doc;
use crate::tag::{AllTags, TagId};

use futures_util::TryStreamExt;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use std::collections::{HashMap, HashSet};

pub async fn load_docs(
    all_tags: &mut AllTags,
    library_path: &str,
) -> anyhow::Result<HashMap<i64, Doc>> {
    let pool = SqlitePool::connect(library_path).await?;
    let mut conn = pool.acquire().await?;
    let mut docs = HashMap::new();

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
                zid,
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
        match docs.get_mut(&item_id) {
            Some(doc) => {
                doc.tags.insert(memex_tag_id.clone());
            }
            None => (), //println!("skipping doc ID {item_id} has no title"),
        };
    }

    Ok(docs)
}
