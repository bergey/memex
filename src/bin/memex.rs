// memex cli

use clap::Parser;
use std::collections::HashMap;
use winnow::Parser as WParser;

use memex::tag::query::*;
use memex::tag::*;
use memex::{Doc, DocId};

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
    let args = Cli::parse();

    let mut all_tags = AllTags::new();
    let mut docs: HashMap<DocId, Doc> = HashMap::new();

    memex::zotero::load_docs(&mut all_tags, &mut docs).await?;

    println!("{} tags", all_tags.len());
    println!("{} titles", docs.len());
    let mut tag_count = 0;
    for (_id, doc) in docs.iter() {
        tag_count += doc.tags.len();
    }
    println!("{} tag-title associations\n", tag_count);

    let query = query
        .parse(&args.query)
        .map_err(|e| anyhow::format_err!("{e}"))?;
    let query = query.compile(&mut all_tags);

    // let rust_tag_id = all_tags.insert("rust".to_string());
    // let tagged = Query::Tag(rust_tag_id);
    // let only = Query::Only(HashSet::from([rust_tag_id]));
    // let query = Query::Function(Operator::And, Vec::from([tagged, only]));
    for (id, doc) in docs.iter() {
        if match_tags(&query, &doc.tags) {
            println!("{:?} {}", id, doc.title);
        }
    }

    Ok(())
}
