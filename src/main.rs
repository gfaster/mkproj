use clap::Parser;
use mkproj::run_main;

fn main() -> anyhow::Result<()> {
    run_main(&mkproj::Cli::parse())
}
