use anyhow::Result;
use clap::Args;

use crate::util::{git_init, write_to_file, enter_nix_shell};

#[derive(Args)]
pub(crate) struct GenericArgs {
    #[command(flatten)]
    common: super::CommonArgs,
}

pub(crate) fn create_generic(args: &GenericArgs) -> Result<()> {
    args.common.begin()?.write_to_shell_dot_nix()?;

    write_to_file(".gitignore", GIT_IGNORE)?;
    git_init()?;

    enter_nix_shell()
}

const GIT_IGNORE: &str = "\
perf.data
perf.data.old
flamegraph.svg
*.fxt
*.fxt.old
/result
";
