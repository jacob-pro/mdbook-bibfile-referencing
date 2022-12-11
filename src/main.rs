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
use clap::{Parser, ValueEnum};
use mdbook::book::{Book, BookItem};
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use pandoc::PandocOutput::ToBuffer;
use pandoc::{InputFormat, MarkdownExtension, OutputFormat, Pandoc, PandocOption};
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
    /// Disables the `link-citations` metadata option
    #[clap(long)]
    disable_links: bool,
    /// Override the input flavour used by Pandoc (not recommended)
    #[clap(long, value_enum, default_value_t = InputFlavour::MarkdownGithub)]
    input_format: InputFlavour,
    /// Override the output flavour used by Pandoc (not recommended)
    #[clap(long, value_enum, default_value_t = OutputFlavour::Gfm)]
    output_format: OutputFlavour,
    #[clap(subcommand)]
    subcommand: Option<SubCommand>,
}

// Currently only these four are compatible with the Pandoc `citation` extension
#[derive(ValueEnum, Copy, Clone)]
enum InputFlavour {
    /// pandoc’s extended markdown (Note: doesn't support mathjax)
    Markdown,
    /// original unextended markdown (Note: doesn't support code blocks)
    MarkdownStrict,
    /// PHP Markdown extra extended markdown
    MarkdownPhpextra,
    /// github extended markdown
    MarkdownGithub,
}

// https://rust-lang.github.io/mdBook/format/markdown.html
// mdBook's parser adheres to the CommonMark specification with some extensions
#[derive(ValueEnum, Copy, Clone)]
enum OutputFlavour {
    /// pandoc’s extended markdown
    Markdown,
    /// original unextended markdown
    MarkdownStrict,
    /// PHP Markdown extra extended markdown
    MarkdownPhpextra,
    /// github extended markdown (deprecated)
    MarkdownGithub,
    /// CommonMark markdown
    Commonmark,
    /// CommonMark markdown with extensions
    CommonmarkX,
    /// GitHub-Flavored Markdown
    Gfm,
}

impl From<InputFlavour> for InputFormat {
    fn from(flavour: InputFlavour) -> Self {
        match flavour {
            InputFlavour::Markdown => InputFormat::Markdown,
            InputFlavour::MarkdownStrict => InputFormat::MarkdownStrict,
            InputFlavour::MarkdownPhpextra => InputFormat::MarkdownPhpextra,
            InputFlavour::MarkdownGithub => InputFormat::MarkdownGithub,
        }
    }
}

impl From<OutputFlavour> for OutputFormat {
    fn from(flavour: OutputFlavour) -> Self {
        match flavour {
            OutputFlavour::Markdown => OutputFormat::Markdown,
            OutputFlavour::MarkdownStrict => OutputFormat::MarkdownStrict,
            OutputFlavour::MarkdownPhpextra => OutputFormat::MarkdownPhpextra,
            OutputFlavour::MarkdownGithub => OutputFormat::MarkdownGithub,
            OutputFlavour::Commonmark => OutputFormat::Commonmark,
            OutputFlavour::CommonmarkX => OutputFormat::CommonmarkX,
            OutputFlavour::Gfm => OutputFormat::Other(String::from("gfm")),
        }
    }
}

fn main() {
    let opts: Opts = Opts::parse();
    match opts.subcommand {
        None => {
            if let Err(e) = handle_preprocessing(opts) {
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

fn handle_preprocessing(opts: Opts) -> anyhow::Result<()> {
    if !opts.bib.exists() {
        bail!("Bib file not found");
    }
    if !opts.csl.exists() {
        bail!("CSL file not found");
    }
    let pre = Bibliography::new(
        opts.bib,
        opts.csl,
        opts.input_format.into(),
        opts.output_format.into(),
        !opts.disable_links,
        builtin_citeproc_support()?,
    );

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
    pub fn new(
        bib: PathBuf,
        csl: PathBuf,
        in_format: InputFormat,
        out_format: OutputFormat,
        link_citations: bool,
        builtin_citeproc: bool,
    ) -> Self {
        let mut p = pandoc::new();
        p.add_option(PandocOption::Csl(csl));
        if builtin_citeproc {
            p.add_option(PandocOption::Citeproc);
        } else {
            p.add_option(PandocOption::Filter("pandoc-citeproc".into()));
        }
        p.set_bibliography(&bib);
        p.set_output(pandoc::OutputKind::Pipe);
        if link_citations {
            p.add_option(PandocOption::Meta(
                "link-citations".into(),
                Some("true".into()),
            ));
        }
        p.set_input_format(
            in_format,
            vec![MarkdownExtension::Citations, MarkdownExtension::Footnotes],
        );
        p.set_output_format(
            out_format,
            vec![
                MarkdownExtension::Footnotes,
                MarkdownExtension::Other("task_lists".into()),
            ],
        );
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
                let path = chapter
                    .path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(String::new);
                eprintln!(
                    "Chapter '{path}' referenced in {}ms",
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
