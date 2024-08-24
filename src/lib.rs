use clap::{Parser, Subcommand};

mod rust;
mod util;
mod generic;
mod python;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Rust(rust::RustArgs),
    Generic(generic::GenericArgs),
    Python(python::PythonArgs),
}

pub fn run_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Rust(args) => rust::create_rust(args)?,
        Commands::Generic(args) => generic::create_generic(args)?,
        Commands::Python(args) => python::create_python(args)?,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn command() {
        Cli::command().debug_assert();
    }
}
