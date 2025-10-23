use core::fmt;

use anyhow::{ensure, Result};
use clap::{Args, ValueEnum};

use crate::util::{enter_nix_shell, file_template::Template, git_init, mk_proj_dir, write_to_file};

#[derive(Args)]
pub(crate) struct OdinArgs {
    /// project name
    #[arg(required = true)]
    name: Box<str>,

    #[arg(short = 'm', long)]
    no_makefile: bool,

    #[arg(short, long, value_enum, default_value = "none")]
    /// setup for use with graphics library
    graphics: Graphics,

    #[arg(short, long)]
    /// packages in nixpkgs
    package: Vec<Box<str>>
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Graphics {
    None,
    Raylib,
}

pub(crate) fn create_odin(args: &OdinArgs) -> Result<()> {
    ensure!(!args.name.contains('-'), "odin packages cannot contain `-`");
    mk_proj_dir(&args.name)?;
    write_to_file(".gitignore", format_args!("{GIT_IGNORE}/{}\n", args.name))?;
    write_to_file("shell.nix", mkshell(args)?)?;
    if !args.no_makefile {
        write_to_file("Makefile", mkmakefile(&args.name))?;
    }
    write_to_file("main.odin",  mkmain(args)?)?;
    git_init()?;

    enter_nix_shell()
}

const MAIN_ODIN: &str = "\
package main

main :: proc () {

}
";

const MAIN_ODIN_RAYLIB_TEMPLATE: &str = include_str!("main_raylib.odin.in");

fn mkmain(args: &OdinArgs) -> Result<String> {
    let ret = match args.graphics {
        Graphics::None => MAIN_ODIN.into(),
        Graphics::Raylib => {
            Template::new(MAIN_ODIN_RAYLIB_TEMPLATE)
                .expect("raylib template invalid")
                .subst("pname", args.name.replace('_', " "))
                .format()
        },
    };
    Ok(ret)
}

const MAKEFILE_TEMPLATE: &str = "\
ODIN_SRC := $(wildcard *.odin)

.PHONY: all
all: {{exe}}

{{exe}}: $(ODIN_SRC)
\todin build .

.PHONY: check c
c: check
check:
\todin check .

.PHONY: build b
b: build
build: {{exe}}

.PHONY: run r
r: run
run: build
\t./{{exe}}

.PHONY: clean
clean:
\trm -f {{exe}}
";

fn mkmakefile(name: &str) -> impl fmt::Display {
    Template::new(MAKEFILE_TEMPLATE).expect("invalid makefile template").subst("exe", name).format()
}

const GIT_IGNORE: &str = "\
perf.data
perf.data.old
flamegraph.svg
*.fxt
*.fxt.old
/result
";

fn mkshell(args: &OdinArgs) -> Result<Box<str>> {
    let mut nix = crate::util::nix::NixBuilder::new();
    nix.add_build_input("odin");
    nix.add_build_inputs(&args.package);

    match args.graphics {
        Graphics::None => (),
        Graphics::Raylib => {
            nix.add_build_inputs(["glfw", "raylib"]);
            nix.add_libraries(["libGL", "xorg.libX11"]);
        },
    }

    Ok(nix.build().into_boxed_str())
}
