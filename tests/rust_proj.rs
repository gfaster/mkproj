mod common;

const BASIC_SCRIPT: &str = r##"
cargo -q build
cargo -q test
"##;

#[test]
fn basic() -> anyhow::Result<()> {
    let name = "rust_basic";
    common::run_test(&format!("rust {name}"))?;
    common::run_bash_script(name, BASIC_SCRIPT)?;
    Ok(())
}
