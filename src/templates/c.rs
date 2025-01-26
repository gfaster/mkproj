use anyhow::{bail, Result};
use clap::{Args, ValueEnum};

use crate::util::{enter_nix_shell, file_template::{template_format, Template}, fmt_fn, git_init, mk_proj_dir, mkdir, write_to_file};

#[derive(Args)]
pub(crate) struct CArgs {
    /// project name
    #[arg(required = true)]
    name: Box<str>,

    /// force the primary language to be c++
    #[arg(short = 'x', long, alias = "force-c++", alias = "force-cpp")]
    force_cxx: bool,

    /// what build system to use
    #[arg(short, long, default_value = "make")]
    build_system: BuildSystem,

    /// standard year -- only integer standard years are supported
    #[arg(long, default_value = "17")]
    std: u32,

    /// C compiler
    #[arg(long, default_value = "gcc")]
    cc: Box<str>,

    /// C++ compiler
    #[arg(long, default_value = "g++")]
    cxx: Box<str>,

    /// name of the executable
    #[arg(short, long)]
    exe: Option<Box<str>>,

    /// packages in nixpkgs
    #[arg(short, long)]
    package: Vec<Box<str>>
}

impl CArgs {
    fn exe(&self) -> &str {
        self.exe.as_deref().unwrap_or(&self.name)
    }
}

pub(crate) fn create_c(args: &CArgs) -> Result<()> {
    if !args.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        bail!("project name should be only ascii alphanumeric and [-_]")
    }
    if !args.exe().chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        bail!("project exe should be only ascii alphanumeric and [-_]")
    }
    mk_proj_dir(&args.name)?;

    write_to_file(".gitignore", 
        template_format!(GIT_IGNORE_TEMPLATE,
            exe = args.name,
            makefile = if args.build_system == BuildSystem::Cmake { "Makefile" } else { "" },
        ))?;
    write_to_file("shell.nix", mkshell(args)?)?;
    git_init()?;
    mkdir("build")?;
    mkdir("src")?;

    if args.force_cxx {
        write_to_file("src/main.cpp", STARTING_CXX_FILE)?;
    } else {
        write_to_file("src/main.c", STARTING_C_FILE)?;
    }

    mk_build(args)?;

    enter_nix_shell()
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum BuildSystem {
    /// Very simple Makefile suitible for tiny projects
    MakeSimple,

    /// Normal Makefile without dependency generation and extra targets, but
    /// generally suitable.
    Make,
    /// Full-fat Makefile
    MakeFull,
    /// Basic cmake template
    Cmake,
}

const STARTING_C_FILE: &str = r#"#include<stdio.h>

int
main(void)
{
    puts("Hello, world!");

    return 0;
}
"#;

const STARTING_CXX_FILE: &str = r#"#include<iostream>

int main() {
    std::cout << "Hello, world!\n";

    return 0;
}
"#;

const MAKE_SIMPLE_TEMPLATE: &str = "\
# preprocessor flags (effectively both C and C++ flags)
CPPFLAGS := -Wall -Wextra -g

# flags passed to C compiler
CFLAGS := -std={{c_std}}

# flags passed to C++ compiler
CXXFLAGS := -std={{cxx_std}}

# Linker flags, excluding libraries
LDFLAGS :=

# Linked libraries flags
LDLIBS := 

.PHONY: all
all: {{exe}}

{{exe}}: $(wildcard src/*.c src/*.cpp)
";

const MAKE_NORMAL_TEMPLATE: &str = "\
# executable name
EXE := {{exe}}

# build directory
BUILD := {{build_dir}}

# Linker to use
LD := {{ld}}

# C compiler
CC := {{cc}}

# C++ compiler
CXX := {{cxx}}

# Linker flags, excluding libraries
LDFLAGS :=

# Linked libraries flags
LDLIBS := 

# preprocessor flags (effectively both C and C++ flags)
CPPFLAGS := -Wall -Wextra -g

# flags passed to C compiler
CFLAGS := -std={{c_std}}

# flags passed to C++ compiler
CXXFLAGS := -std={{cxx_std}}

OBJECTS := \
    $(patsubst src/%.c, $(BUILD)/%.o, $(wildcard src/*.c)) \
    $(patsubst src/%.cpp, $(BUILD)/%.o, $(wildcard src/*.cpp))

.DEFAULT_GOAL = all

.PHONY: all
all: $(BUILD)/$(EXE)

$(BUILD):
\tmkdir -p $(BUILD)

$(BUILD)/%.o: src/%.c
$(BUILD)/%.o: src/%.c
\t$(CC) $(CPPFLAGS) $(CFLAGS) -c $^ -o $@

$(BUILD)/%.o: src/%.cpp
$(BUILD)/%.o: src/%.cpp
\t$(CXX) $(CPPFLAGS) $(CXXFLAGS) -c $^ -o $@

$(BUILD)/$(EXE): $(OBJECTS)
\t$(LD) $(LDFLAGS) $^ $(LDLIBS) -o $@


.PHONY: clean
clean:
\t$(RM) $(OBJECTS)
\t$(RM) $(BUILD)/$(EXE)
";

const CMAKE_TEMPLATE: &str = "\
    cmake_minimum_required(VERSION 3.10)
    project({{name}})

    set(CMAKE_CXX_STANDARD {{cmake_cxx_std}})
    set(CMAKE_CXX_STANDARD_REQUIRED True)

    set(CMAKE_SOURCE_DIR src)

    add_executable({{exe}} src/main.{{ext}})
";

const GIT_IGNORE_TEMPLATE: &str = "\
*.o
*.d
/{{exe}}
/build
perf.data
perf.data.old
flamegraph.svg
*.fxt
*.fxt.old
/result
{{makefile}}
";

fn mkshell(args: &CArgs) -> Result<Box<str>> {
    let mut nix = crate::util::nix::NixBuilder::new();
    nix.add_build_inputs(&args.package);
    if args.build_system == BuildSystem::Cmake {
        nix.add_build_input("cmake");
    }

    if (args.cxx.starts_with("clang") || args.cc.starts_with("clang")) && !args.package.iter().any(|p| p.contains("llvm") || p.contains("clang")) {
        nix.add_build_input("clang");
    }

    Ok(nix.build().into_boxed_str())
}

fn mk_build(args: &CArgs) -> Result<()> {
    let template = match args.build_system {
        BuildSystem::MakeSimple => MAKE_SIMPLE_TEMPLATE,
        BuildSystem::Make => MAKE_NORMAL_TEMPLATE,
        BuildSystem::MakeFull => todo!("MakeFull"),
        BuildSystem::Cmake => CMAKE_TEMPLATE,
    };
    let mut template = Template::new(template).unwrap();
    template
        .subst("exe", args.exe())
        .subst("name", &args.name)
        .subst("c_std", fmt_fn(|f| write!(f, "c{}", args.std)))
        .subst("cxx_std", fmt_fn(|f| write!(f, "c++{}", args.std)))
        .subst("cmake_cxx_std", args.std)
        .subst("build_dir", "build")
        .subst("ext", if args.force_cxx { "cpp" } else { "c" })
        .subst("cc", &args.cc)
        .subst("cxx", &args.cxx)
        .subst("ld", if args.force_cxx { &args.cxx } else { &args.cc })
    ;
    let file = match args.build_system {
        BuildSystem::MakeSimple |
        BuildSystem::Make |
        BuildSystem::MakeFull => "Makefile",
        BuildSystem::Cmake => "CMakeLists.txt",
    };
    write_to_file(file, template.format())?;
    Ok(())
}
