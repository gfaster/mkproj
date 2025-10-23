use clap::ValueEnum;
use anyhow::Result;
use clap::Args;

use crate::util::{git_init, write_to_file, enter_nix_shell};

/// Write basic documents with markdown and create pdfs
#[derive(Args)]
pub(crate) struct DocumentsArgs {
    #[command(flatten)]
    common: super::CommonArgs,

    #[arg(short, long, value_enum, default_value = "pdflatex")]
    engine: Engine,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Engine {
    None,
    Pdflatex,
    Xelatex,
}

pub(crate) fn create_documents(args: &DocumentsArgs) -> Result<()> {
    let mut nix = args.common.begin()?;

    nix.write_to_shell_dot_nix()?;

    write_to_file(".gitignore", GIT_IGNORE)?;
    git_init()?;

    enter_nix_shell()
}

const GIT_IGNORE: &str = "\
*.pdf
*.aux
*.log
";

const MAKEFILE_TEMPLATE: &str = "\
.PHONY: all
all: {{pdf}}

%.pdf: %.md
\t{{pandoc-cmd}}

.PHONY: clean
clean:
\trm -f {{pdf}}
";
