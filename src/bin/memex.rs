// memex cli

use clap::Parser;
use std::collections::HashMap;

use memex::tag::query::*;
use memex::tag::*;
use memex::Doc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command()]
    query: String,
    #[arg(long = "stats", help = "print stats about library size")]
    stats: bool,
    #[arg(short = 'v', long = "verbose", help = "print what we're doing")]
    verbose: bool,
    // #[arg(long="library")]
    // libraries: Vec<String>
}

/// DB path -> ID in DB -> Doc
type Docs = HashMap<String, HashMap<i64, Doc>>;

#[tokio::main()]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    // fail fast if we can't parse it
    let query = parse_query(&args.query)?;

    let mut all_tags = AllTags::new();
    let mut docs: Docs = HashMap::new();

    // TODO --library arg
    if let Ok(home) = std::env::var("HOME") {
        let library_path = format!("{home}/Zotero/zotero.sqlite");
        docs.insert(
            library_path.clone(),
            memex::zotero::load_docs(&mut all_tags, &library_path).await?,
        );
        for glob_path in glob::glob(&format!("{home}/Calibre/**/metadata.db"))? {
            let ok_path = glob_path?;
            let library_path = ok_path
                .to_str()
                .ok_or(anyhow::anyhow!("path is not unicode"))?;
            if args.verbose {
                println!("loading from {library_path}");
            }
            docs.insert(
                library_path.to_string(),
                memex::calibre::load_docs(&mut all_tags, library_path).await?,
            );
        }
    }

    if args.stats {
        print_stats(&all_tags, &docs);
    }

    let query = query.compile(&mut all_tags);
    for (path, docs) in docs.iter() {
        let mut first = true;
        for (id, doc) in docs.iter() {
            if match_tags(&query, &doc.tags) {
                if first {
                    println!("\n{path}");
                    first = false;
                }
                println!("  {} {}", id, doc.title);
            }
        }
    }

    Ok(())
}

fn print_stats(all_tags: &AllTags, libraries: &Docs) {
    println!("{} tags", all_tags.len());
    let mut titles_count = 0;
    let mut links_count = 0;

    for (_path, docs) in libraries.iter() {
        titles_count += docs.len();
        for (_id, doc) in docs.iter() {
            links_count += doc.tags.len();
        }
    }

    println!("{} titles", titles_count);
    println!("{} tag-title associations\n", links_count);
}
