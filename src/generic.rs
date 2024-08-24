use anyhow::{bail, Result};
use clap::Args;

use crate::util::{git_init, mk_proj_dir, write_to_file, enter_nix_shell};

#[derive(Args)]
pub(crate) struct GenericArgs {
    /// project name
    #[arg(required = true)]
    name: Box<str>,

    #[arg(short, long)]
    /// packages in nixpkgs
    package: Vec<Box<str>>
}

pub(crate) fn create_generic(args: &GenericArgs) -> Result<()> {
    if !args.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        bail!("project name should be only ascii alphanumeric and [-_]")
    }
    mk_proj_dir(&args.name)?;
    write_to_file(".gitignore", GIT_IGNORE)?;
    write_to_file("shell.nix", mkshell(args)?)?;
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

fn mkshell(args: &GenericArgs) -> Result<Box<str>> {
    let mut nix = crate::util::nix::NixBuilder::new();
    nix.add_build_inputs(&args.package);

    Ok(nix.build().into_boxed_str())
}
