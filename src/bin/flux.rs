use std::path::PathBuf;

use pdf::{
    build::*,
    content::{Cmyk, Color, Matrix, Op},
    error::PdfError,
    file::FileOptions,
    font::{Font, FontData, TFont},
    object::*,
    primitive::{Name, PdfString},
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file
    #[command()]
    input: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let old_file = FileOptions::cached().open(&args.input)?;
    let old_page = old_file.get_page(0)?;
    let mut builder = PdfBuilder::new(FileOptions::cached());
    let importer = Importer::new(old_file.resolver(), &mut builder.storage);

    if let Some(content) = &old_page.contents {
        for op in content.operations(&importer)? {
            use pdf::content::Op::*;
            match op {
                TextDraw { text } => print!("{:?}", text),
                TextNewline => println!(""),
                MoveTextPosition { translation } => {
                    if translation.x < 0.0 && translation.y < 0.0 {
                        println!("");
                    } else if translation.x > 0.0 && translation.y > 0.0 {
                        println!("");
                        println!("*** NEW COLUMN ***");
                    }
                }
                _ => println!("{:?}", op),
            }
        }
    }

    return Ok(());
}
