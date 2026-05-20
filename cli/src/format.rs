use std::ffi::OsStr;
use std::io::Write;
use std::path::Path;

use super::Doc;

pub enum Style {
    Html,
    Org,
    Stdout,
}

type Result = std::io::Result<()>;

impl Style {
    pub fn from_path(path: &Path) -> Style {
        use Style::*;
        match path.extension() {
            None => Stdout,
            Some(ext) => {
                if ext == OsStr::new("org") {
                    Org
                } else if ext == OsStr::new("html") {
                    Html
                } else {
                    Stdout
                }
            }
        }
    }

    pub fn doc(&self, doc: &Doc, output: &mut dyn Write) -> Result {
        use Style::*;
        match self {
            Html => writeln!(
                output,
                "    <li><a href='{}'>{}</a></li>",
                doc.link, doc.title
            ),
            Org => writeln!(output, "- [[{}][{}]]", doc.link, doc.title),
            Stdout => writeln!(output, "  {}", doc.title),
        }
    }

    pub fn library(&self, path: &str, output: &mut dyn Write) -> Result {
        use Style::*;
        match self {
            Html => writeln!(output, "    <h1>{path}</h1>"),
            Org => writeln!(output, "* {path}"),
            Stdout => writeln!(output, "\n{path}"),
        }
    }

    pub fn header(&self, output: &mut dyn Write) -> Result {
        use Style::*;
        match self {
            Html => writeln!(output, "<!DOCTYPE html>\n<html>\n  <body>"),
            Org => Ok(()),
            Stdout => Ok(()),
        }
    }

    pub fn footer(&self, output: &mut dyn Write) -> Result {
        use Style::*;
        match self {
            Html => writeln!(output, "  </body>\n</html>"),
            Org => Ok(()),
            Stdout => Ok(()),
        }
    }
}
