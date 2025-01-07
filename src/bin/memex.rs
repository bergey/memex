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
    query: Option<String>,
    #[arg(long = "matches", help = "print matching documents (default)")]
    matches: bool,
    #[arg(long = "stats", help = "print stats about library size")]
    stats: bool,
    #[arg(
        long = "top",
        help = "print stats on this many common tags",
        default_value_t = 5
    )]
    top: usize,
    #[arg(short = 'v', long = "verbose", help = "print what we're doing")]
    verbose: bool,
    #[arg(
        long = "library",
        help = "path to library; suppresses default search locations"
    )]
    libraries: Vec<String>,
}

impl Cli {
    pub fn init() -> Self {
        let mut args = Cli::parse();
        if !args.matches && !args.stats {
            args.matches = true;
        }
        args
    }
}

/// DB path -> [Doc]
type Docs = HashMap<String, Vec<Doc>>;

#[tokio::main()]
async fn main() -> anyhow::Result<()> {
    let args = Cli::init();
    // fail fast if we can't parse it
    let query = match args.query {
        Some(q) => Some(parse_query(&q)?),
        None => None,
    };

    let mut all_tags = AllTags::new();
    let mut docs: Docs = HashMap::new();

    let mut libraries = args.libraries.clone();
    if libraries.is_empty() {
        if let Ok(home) = std::env::var("HOME") {
            let path = format!("{home}/Zotero/zotero.sqlite");
            if std::path::Path::new(&path).exists() {
                libraries.push(path);
            }
            for glob_path in glob::glob(&format!("{home}/Calibre/**/metadata.db"))? {
                let ok_path = glob_path?;
                let library_path = ok_path
                    .to_str()
                    .ok_or(anyhow::anyhow!("path is not unicode"))?;
                libraries.push(library_path.to_string());
            }
        }
    }

    for library_path in libraries {
        if !std::path::Path::new(&library_path).exists() {
            println!("skipping {library_path}, file does not exist");
            continue;
        }
        if args.verbose {
            println!("loading from {library_path}");
        }
        let new_docs = if library_path.ends_with("zotero.sqlite") {
            memex::zotero::load_docs(&mut all_tags, &library_path).await?
        } else if library_path.ends_with("metadata.db") {
            memex::calibre::load_docs(&mut all_tags, &library_path).await?
        } else {
            println!("skipping {library_path}, unknown format");
            Vec::new()
        };
        docs.insert(library_path.clone(), new_docs);
    }

    if args.stats {
        print_stats(&all_tags, &docs);
    }

    let mut tag_counts = memex::stats::TagCounts::new();

    if let Some(query) = query {
        let query = query.compile(&mut all_tags);
        for (path, docs) in docs.iter() {
            let mut first = true;
            for doc in docs.iter() {
                if match_tags(&query, &doc.tags) {
                    if args.matches {
                        if first {
                            println!("\n{path}");
                            first = false;
                        }
                        println!("  {} {}", "todo", doc.title);
                    }
                    if args.stats {
                        tag_counts.count(&doc.tags);
                    }
                }
            }
        }
    } else if args.stats {
        for (_, docs) in docs.iter() {
            for doc in docs.iter() {
                tag_counts.count(&doc.tags);
            }
        }
    }

    if args.stats {
        let mut tag_counts = tag_counts.to_vec();
        tag_counts.sort_by(|(_, a), (_, b)| b.cmp(a));
        for (t, ct) in &tag_counts[..args.top] {
            let tag = all_tags.name(*t).unwrap();
            println!("{tag}: {ct}");
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
        for doc in docs.iter() {
            links_count += doc.tags.len();
        }
    }

    println!("{} titles", titles_count);
    println!("{} tag-title associations\n", links_count);
}
