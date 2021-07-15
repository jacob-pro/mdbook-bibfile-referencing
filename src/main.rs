use clap::Clap;
use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use pandoc::PandocOutput::ToBuffer;
use std::io;
use std::path::PathBuf;
use std::process;

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
    let pre = Bibliography::new(bib, csl);

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
    bib: PathBuf,
    csl: PathBuf,
}

impl Bibliography {
    pub fn new(bib: PathBuf, csl: PathBuf) -> Self {
        Self { bib, csl }
    }
}

impl Preprocessor for Bibliography {
    fn name(&self) -> &str {
        "mdbook-bibfile-referencing"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                let mut p = pandoc::new();
                p.set_input(pandoc::InputKind::Pipe(chapter.content.clone()));
                p.add_option(pandoc::PandocOption::Filter("pandoc-citeproc".into()));
                p.add_option(pandoc::PandocOption::Csl(self.csl.clone()));
                p.set_bibliography(&self.bib);
                p.set_output(pandoc::OutputKind::Pipe);
                p.set_output_format(pandoc::OutputFormat::MarkdownStrict, vec![]);
                if let ToBuffer(x) = p.execute().unwrap() {
                    chapter.content = x;
                }
            }
        });
        Ok(book)
    }
}

