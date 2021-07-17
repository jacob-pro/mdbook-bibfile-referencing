#[cfg(test)]
mod test;

use clap::Clap;
use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use pandoc::PandocOutput::ToBuffer;
use pandoc::{Pandoc, PandocOption};
use std::io;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use version_compare::Version;

#[derive(Clap)]
#[allow(unused)]
struct SupportsSubCommand {
    #[clap(about = "Check whether a renderer is supported by this preprocessor")]
    renderer: String,
}

#[derive(Clap)]
enum SubCommand {
    Supports(SupportsSubCommand),
}

#[derive(Clap)]
#[clap(
    version = "0.1.0",
    author = "Jacob Halsey <jacob@jhalsey.com>",
    about = "An mdBook preprocessor to add bibfile referencing to each page"
)]
struct Opts {
    bib: PathBuf,
    csl: PathBuf,
    #[clap(subcommand)]
    subcommand: Option<SubCommand>,
}

fn main() {
    let opts: Opts = Opts::parse();
    match opts.subcommand {
        None => {
            if let Err(e) = handle_preprocessing(opts.bib, opts.csl) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        Some(cmd) => match cmd {
            SubCommand::Supports(_) => {
                process::exit(0);
            }
        },
    }
}

fn handle_preprocessing(bib: PathBuf, csl: PathBuf) -> Result<(), Error> {
    if !bib.exists() {
        Error::msg("Bib file not found");
    }
    if !csl.exists() {
        Error::msg("CSL file not found");
    }
    let pre = Bibliography::new(bib, csl, builtin_citeproc_support()?);

    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }
    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}

pub struct Bibliography {
    pandoc: Pandoc,
}

impl Bibliography {
    pub fn new(bib: PathBuf, csl: PathBuf, builtin_citeproc: bool) -> Self {
        let mut p = pandoc::new();
        p.add_option(PandocOption::Csl(csl));
        if builtin_citeproc {
            p.add_option(PandocOption::Citeproc);
        } else {
            p.add_option(PandocOption::Filter("pandoc-citeproc".into()));
        }
        p.set_bibliography(&bib);
        p.set_output(pandoc::OutputKind::Pipe);
        p.set_output_format(pandoc::OutputFormat::MarkdownStrict, vec![]);
        Self { pandoc: p }
    }
}

impl Preprocessor for Bibliography {
    fn name(&self) -> &str {
        "mdbook-bibfile-referencing"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                let mut p = self.pandoc.clone();
                p.set_input(pandoc::InputKind::Pipe(chapter.content.clone()));
                if let ToBuffer(x) = p.execute().unwrap() {
                    chapter.content = x;
                }
            }
        });
        Ok(book)
    }
}

pub fn builtin_citeproc_support() -> Result<bool, Error> {
    let output = Command::new("pandoc")
        .arg("--version")
        .output()
        .map_err(|_| Error::msg("Failed to call pandoc - is it installed?"))?;
    if !output.status.success() {
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        Err(Error::msg(format!("mdbook failed to clean: {}", stderr)))
    } else {
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        let version = stdout
            .lines()
            .next()
            .ok_or(Error::msg("Pandoc version error"))?
            .replace("pandoc ", "");
        let installed = Version::from(&version).ok_or(Error::msg(format!(
            "Failed to parse pandoc version: {}",
            version
        )))?;
        let required = Version::from("2.11.0").unwrap();
        return Ok(installed >= required);
    }
}
