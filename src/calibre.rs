use super::Doc;
use crate::tag::{AllTags, TagId};

use futures_util::TryStreamExt;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub async fn load_docs(all_tags: &mut AllTags, library_path: &str) -> anyhow::Result<Vec<Doc>> {
    // TODO replace space by _
    let library_name = Path::new(library_path).parent().map_or("", |path| {
        path.components()
            .last()
            .map_or("", |c| c.as_os_str().to_str().unwrap_or(""))
    });
    let pool = SqlitePool::connect(library_path).await?;
    let mut conn = pool.acquire().await?;
    let mut docs: HashMap<i64, Doc> = HashMap::new();

    // tags
    let calibre_tags = {
        let mut calibre_tags: HashMap<i64, TagId> = HashMap::new();
        let mut rows = sqlx::query("select id, name from tags").fetch(&mut *conn);
        while let Some(row) = rows.try_next().await? {
            let cid = row.try_get("id")?;
            let name = row.try_get("name")?;
            let memex_id = all_tags.insert(name);
            calibre_tags.insert(cid, memex_id);
        }
        calibre_tags
    };

    //titles
    {
        let mut rows = sqlx::query("select id, title from books").fetch(&mut *conn);
        while let Some(row) = rows.try_next().await? {
            let id = row.try_get("id")?;
            let title = row.try_get("title")?;
            docs.insert(
                id,
                Doc {
                    title: title,
                    link: format!("calibre://show-book/{library_name}/{id}"),
                    // https://manual.calibre-ebook.com/url_scheme.html#show-a-specific-book-in-calibre
                    tags: HashSet::new(),
                },
            );
        }
    }

    // tag the documents
    let mut rows = sqlx::query("select book, tag from books_tags_link").fetch(&mut *conn);
    while let Some(row) = rows.try_next().await? {
        let book_id = row.try_get("book")?;
        let c_tag_id = row.try_get("tag")?;
        let memex_tag_id = calibre_tags
            .get(&c_tag_id)
            .ok_or(anyhow::anyhow!("tag ID violates foreign key constraint"))?;
        match docs.get_mut(&book_id) {
            Some(doc) => {
                doc.tags.insert(memex_tag_id.clone());
            }
            None => (), //println!("skipping doc ID {item_id} has no title"),
        };
    }

    Ok(docs.into_values().collect())
}
