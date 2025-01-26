use clap::{Parser, Subcommand};

static TEST_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(cfg!(test));

#[doc(hidden)]
pub fn enable_test_mode() {
    TEST_MODE.store(true, std::sync::atomic::Ordering::SeqCst);
}

fn in_test_mode() -> bool {
    TEST_MODE.load(std::sync::atomic::Ordering::SeqCst)
}

pub mod util;
mod templates;
use templates::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Rust(rust::RustArgs),
    Generic(generic::GenericArgs),
    Python(python::PythonArgs),
    C(c::CArgs),
}

pub fn run_main(cli: &Cli) -> anyhow::Result<()> {
    match &cli.command {
        Commands::Rust(args) => rust::create_rust(args)?,
        Commands::Generic(args) => generic::create_generic(args)?,
        Commands::Python(args) => python::create_python(args)?,
        Commands::C(args) => c::create_c(args)?,
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
