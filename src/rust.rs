use clap::ValueEnum;
use anyhow::{bail, Result};
use clap::Args;

use crate::util::{enter_nix_shell, git_init, mk_proj_dir, mkdir, touch_new, write_to_file};

#[derive(Args)]
pub(crate) struct RustArgs {
    /// crate name
    #[arg(required = true)]
    name: Box<str>,

    /// create a binary app
    #[arg(short, long, group = "crate_type")]
    bin: bool,
    /// create a library
    #[arg(short, long, group = "crate_type")]
    lib: bool,

    /// which Rust toolchain is used
    #[arg(short, long, default_value = "stable")]
    toolchain: Toolchain,

    #[arg(short, long)]
    /// packages in nixpkgs
    package: Vec<Box<str>>
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Toolchain {
    Stable,
    Nightly,
    Beta,
}

pub(crate) fn create_rust(args: &RustArgs) -> Result<()> {
    if !args.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        bail!("crate name should be only ascii alphanumeric and underscores (no dashes)")
    }
    mk_proj_dir(&args.name)?;
    write_to_file(".gitignore", GIT_IGNORE)?;
    write_to_file("shell.nix", mkshell(args)?)?;
    write_to_file("rust-toolchain.toml", mktoolchain(args))?;
    write_to_file("Cargo.toml", mkcargo(args))?;
    mkdir("tests")?;

    {
        mkdir("src")?;
        // always create lib.rs for use in integration tests and to allow doc tests
        touch_new("src/lib.rs")?;

        if args.bin {
            write_to_file("src/main.rs", mkmain(args))?;
        }
    }

    git_init()?;

    enter_nix_shell()
}

fn mkmain(args: &RustArgs) -> Box<str> {
    assert!(args.bin);
    let name = &args.name;
    // the extern crate isn't technically needed, but it's there as a reminder
    format!("extern crate {name};\n\n{MAIN_FN}").into_boxed_str()
}

const MAIN_FN: &str = r#"fn main() {
    println!("hello, world!");
}"#;



const GIT_IGNORE: &str = "\
perf.data
perf.data.old
flamegraph.svg
*.fxt
*.fxt.old
/target
";

fn mkcargo(args: &RustArgs) -> Box<str> {
    format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]"#, args.name).into_boxed_str()
}

fn mktoolchain(args: &RustArgs) -> Box<str> {
    let channel = match args.toolchain {
        Toolchain::Stable => "stable",
        Toolchain::Nightly => "nightly",
        Toolchain::Beta => "beta",
    };
format!(r#"[toolchain]
channel = "{channel}"
"#).into_boxed_str()
}

fn mkshell(args: &RustArgs) -> Result<Box<str>> {
    let mut nix = crate::util::nix::NixBuilder::new();
    nix
        .rec()
        .add_letvar_expr("overrides", "(builtins.fromTOML (builtins.readFile ./rust-toolchain.toml))")?
        .add_letvar_expr("libPath", "\
            with pkgs; lib.makeLibraryPath [\n  \
            # load external libraries that you need in your rust project here\n\
            ]"
        )?
        .add_expr_attribute("RUSTC_VERSION", "overrides.toolchain.channel")?
        .add_expr_attribute_comment("LIBCLANG_PATH", "pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ]", "https://github.com/rust-lang/rust-bindgen#environment-variables")?
        .add_string_attribute("shellHook", "\n  \
            export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin\n  \
            export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/\n  \
            ")?
        .add_expr_attribute_comment("RUSTFLAGS", RUSTFLAGS, "Add precompiled library to rustc search path")?
        .add_expr_attribute("LD_LIBRARY_PATH", "libPath")?
        .add_expr_attribute_comment("BINDGEN_EXTRA_CLANG_ARGS", BINDGEN_ARGS, "Add glibc, clang, glib, and other headers to bindgen search path")?
        .add_build_inputs(["clang", "llvmPackages_17.bintools", "rustup"])
        .add_build_inputs(&args.package)
    ;

    Ok(nix.build().into_boxed_str())
}

const RUSTFLAGS: &str = r"(builtins.map (a: ''-L ${a}/lib'') [
  # add libraries here (e.g. pkgs.libvmi)
])";

const BINDGEN_ARGS: &str = r#"
# Includes normal include path
(builtins.map (a: ''-I"${a}/include"'') [
  # add dev libraries here (e.g. pkgs.libvmi.dev)
  pkgs.glibc.dev
])
# Includes with special directory paths
++ [
  ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
  ''-I"${pkgs.glib.dev}/include/glib-2.0"''
  ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
]"#;


#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn shell_is_ok() {
        // this tests that NixBuilder works
        let cli = crate::Cli::try_parse_from(
            "mkproj rust test_proj".split_whitespace()
        ).map_err(|e| println!("{e}")).unwrap();

        let crate::Commands::Rust(ref args) = cli.command else {
            panic!("wrong command")
        };

        let expected = include_str!("../shell.nix");
        let actual = mkshell(args).unwrap();
        std::fs::write("testoutput.nix", actual.as_bytes()).unwrap();
        assert!(expected == &*actual, "expected:\n{expected}\n\nactual:\n{actual}");
    }
}
