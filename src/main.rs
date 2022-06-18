//!
//! An mdBook preprocessor that uses Pandoc to add referencing to each chapter from a bibfile.
//!
//! Usage: In your book.toml file define the preprocessor command with the paths
//! to your .bib and .csl files.
//! ```
//! [preprocessor.bibliography]
//! command = "mdbook-bibfile-referencing bibliography.bib ieee.csl"
//! ```
//!
//! See the [Pandoc Citeproc guide](https://pandoc.org/demo/example19/Extension-citations.html)
//! for how to use references in your book's markdown source.
//!

#[cfg(test)]
mod test;

use anyhow::{bail, Context};
use clap::Parser;
use mdbook::book::{Book, BookItem};
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use pandoc::PandocOutput::ToBuffer;
use pandoc::{Pandoc, PandocOption};
use std::io;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use std::time::Instant;
use version_compare::Version;

#[derive(Parser)]
#[allow(unused)]
struct SupportsSubCommand {
    /// Check whether a renderer is supported by this preprocessor
    renderer: String,
}

#[derive(Parser)]
enum SubCommand {
    Supports(SupportsSubCommand),
}

#[derive(Parser)]
#[clap(author, version, about)]
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
                eprintln!("{:#}", e);
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

fn handle_preprocessing(bib_file: PathBuf, csl_file: PathBuf) -> anyhow::Result<()> {
    if !bib_file.exists() {
        bail!("Bib file not found");
    }
    if !csl_file.exists() {
        bail!("CSL file not found");
    }
    let pre = Bibliography::new(bib_file, csl_file, builtin_citeproc_support()?);

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

struct Bibliography {
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

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> anyhow::Result<Book> {
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                let now = Instant::now();
                let mut p = self.pandoc.clone();
                p.set_input(pandoc::InputKind::Pipe(chapter.content.clone()));
                if let ToBuffer(x) = p.execute().unwrap() {
                    chapter.content = x;
                }
                let name = chapter.content.lines().next().unwrap_or("");
                eprintln!(
                    "Chapter '{name}' referenced in {}ms",
                    now.elapsed().as_millis()
                );
            }
        });
        Ok(book)
    }
}

fn builtin_citeproc_support() -> anyhow::Result<bool> {
    let output = Command::new("pandoc")
        .arg("--version")
        .output()
        .context("Failed to call pandoc - is it installed?")?;
    if !output.status.success() {
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        bail!(format!("Failed to get pandoc version: {}", stderr));
    } else {
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        let version = stdout
            .lines()
            .next()
            .context("Pandoc version error")?
            .split_whitespace()
            .nth(1)
            .context(format!("Failed to parse pandoc version: {}", stdout))?;
        let installed = Version::from(version)
            .context(format!("Failed to parse pandoc version: {}", version))?;
        let required = Version::from("2.11.0").unwrap();
        Ok(installed >= required)
    }
}
