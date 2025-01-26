use clap::ValueEnum;
use anyhow::{bail, Result};
use clap::Args;

use crate::util::{git_init, mk_proj_dir, write_to_file, enter_nix_shell};

#[derive(Args)]
pub(crate) struct PythonArgs {
    /// project name
    #[arg(required = true)]
    name: Box<str>,

    /// python version
    #[arg(short = 'y', long, default_value = "3.12")]
    python_version: PyVersion,

    #[arg(short = 'i', long)]
    /// packages in nixpkgs
    python_package: Vec<Box<str>>,

    #[arg(short, long)]
    /// packages in nixpkgs
    package: Vec<Box<str>>
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum PyVersion {
    #[value(name = "3.8")]
    Py3_8,
    #[value(name = "3.9")]
    Py3_9,
    #[value(name = "3.10")]
    Py3_10,
    #[value(name = "3.11")]
    Py3_11,
    #[value(name = "3.12")]
    Py3_12,
    #[value(name = "3.13")]
    Py3_13,
    #[value(name = "3.14")]
    Py3_14,
    #[value(name = "3.15")]
    Py3_15,
    #[value(name = "3.16")]
    Py3_16,
}

pub(crate) fn create_python(args: &PythonArgs) -> Result<()> {
    if !args.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        bail!("project name should be only ascii alphanumeric and [-_]")
    }
    mk_proj_dir(&args.name)?;
    write_to_file(".gitignore", GIT_IGNORE)?;
    write_to_file("shell.nix", mkshell(args)?)?;
    git_init()?;

    enter_nix_shell()
}

const GIT_IGNORE: &str = "
perf.data
perf.data.old
flamegraph.svg
*.fxt
*.fxt.old
/result
__pycache__/
*.py[cod]
build/
";

fn mkshell(args: &PythonArgs) -> Result<Box<str>> {
    let version = match args.python_version {
        PyVersion::Py3_8 => 38,
        PyVersion::Py3_9 => 39,
        PyVersion::Py3_10 => 310,
        PyVersion::Py3_11 => 311,
        PyVersion::Py3_12 => 312,
        PyVersion::Py3_13 => 313,
        PyVersion::Py3_14 => 314,
        PyVersion::Py3_15 => 315,
        PyVersion::Py3_16 => 316,
    };
    let mut nix = crate::util::nix::NixBuilder::new();
    nix.add_letvar_expr("py", format!("pkgs.python{version}Packages"))?
        .add_build_input("py.python")
        .add_build_inputs(args.python_package.iter().map(|p| format!("py.{p}")))
        .add_build_inputs(&args.package);

    Ok(nix.build().into_boxed_str())
}
