use clap::{Parser, Subcommand};

pub mod rust;
pub mod util;

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
}

pub fn run_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Rust(args) => rust::create_rust(args)?,
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
