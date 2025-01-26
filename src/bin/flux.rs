use std::path::PathBuf;

use pdf::{
    build::*,
    content::{deep_clone_op, Op},
    error::PdfError,
    file::FileOptions,
    object::*,
    primitive::PdfString,
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file
    #[command()]
    input: PathBuf,

    /// Output file
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let old_file = FileOptions::cached().open(&args.input)?;

    let mut builder = PdfBuilder::new(FileOptions::cached());
    let mut importer = Importer::new(old_file.resolver(), &mut builder.storage);

    let new_pages = split_pages(&mut importer, &mut old_file.pages())?;
    let catalog = CatalogBuilder::from_pages(new_pages);

    let info = InfoDict {
        title: Some(PdfString::from("test")),
        ..InfoDict::default()
    };
    let data = builder.info(info).build(catalog)?;

    std::fs::write(&args.output, data)?;

    return Ok(());
}

/// per-page variables
struct PageProgress {
    ops: Vec<Op>,
    resources: Resources,
}

impl PageProgress {
    pub fn new() -> Self {
        PageProgress {
            ops: Vec::new(),
            resources: Resources::default(),
        }
    }
}

fn split_pages(
    importer: &mut (impl Cloner + Resolve),
    old_pages: impl Iterator<Item = Result<PageRc, PdfError>>,
) -> anyhow::Result<Vec<PageBuilder>> {
    let mut pages = Vec::new();
    let mut new = PageProgress::new();

    for old_page in old_pages {
        let old_page = old_page?;
        // resources of this page or inherited from parent
        let old_resources = &**old_page.resources()?.data();

        if let Some(content) = &old_page.contents {
            for op in content.operations(importer)? {
                use Op::*;
                match op {
                    MoveTextPosition { translation }
                    // new column
                    if translation.x > 0.0 && translation.y > 0.0 =>
                    {
                        pages
                            .push(new_page(&old_page, importer, new)?);
                        new = PageProgress::new();
                    },
                    // serialize not yet implemented in pdf crate
                    InlineImage{..} => eprintln!("skipping inline image"),
                    _ =>
                        match deep_clone_op(&op, importer, old_resources, &mut new.resources) {
                            Ok(new_op) => new.ops.push(new_op),
                            Err(err) => eprintln!("{:?}: {err}", op),
                        },
                    }
            }
        }
        pages.push(new_page(&old_page, importer, new)?);
        new = PageProgress::new();
    }
    Ok(pages)
}

fn new_page(
    old_page: &Page,
    cloner: &mut impl Cloner,
    new: PageProgress,
) -> Result<PageBuilder, PdfError> {
    Ok(PageBuilder {
        ops: new.ops,
        media_box: Some(old_page.media_box()?),
        crop_box: Some(old_page.crop_box()?),
        trim_box: old_page.trim_box,
        resources: new.resources,
        rotate: old_page.rotate,
        metadata: old_page.metadata.deep_clone(cloner)?,
        lgi: old_page.lgi.deep_clone(cloner)?,
        vp: old_page.vp.deep_clone(cloner)?,
        other: old_page.other.deep_clone(cloner)?,
    })
}
