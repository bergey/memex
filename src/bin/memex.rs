// memex cli

use clap::Parser;
use std::collections::HashMap;

use memex::tag::query::*;
use memex::tag::*;
use memex::{Doc, DocId};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command()]
    query: String,
    #[arg(long = "stats", help = "print stats about library size")]
    stats: bool,
    // #[arg(long="library")]
    // libraries: Vec<String>
}

#[tokio::main()]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    // fail fast if we can't parse it
    let query = parse_query(&args.query)?;

    let mut all_tags = AllTags::new();
    let mut docs: HashMap<DocId, Doc> = HashMap::new();

    // TODO --library arg
    if let Ok(home) = std::env::var("HOME") {
        memex::zotero::load_docs(&mut all_tags, &mut docs, &format!("{home}/Zotero/zotero.sqlite")).await?;
        memex::calibre::load_docs(&mut all_tags, &mut docs, &format!("{home}/Calibre/metadata.db")).await?;
    }

    if args.stats {
        print_stats(&all_tags, &docs);
    }

    let query = query.compile(&mut all_tags);
    for (id, doc) in docs.iter() {
        if match_tags(&query, &doc.tags) {
            println!("{:?} {}", id, doc.title);
        }
    }

    Ok(())
}

fn print_stats(all_tags: &AllTags, docs: &HashMap<DocId, Doc>) {
    println!("{} tags", all_tags.len());
    println!("{} titles", docs.len());
    let mut tag_count = 0;
    for (_id, doc) in docs.iter() {
        tag_count += doc.tags.len();
    }
    println!("{} tag-title associations\n", tag_count);
}
