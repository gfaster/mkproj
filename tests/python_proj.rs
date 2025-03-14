mod common;

#[test]
fn python_basic() -> anyhow::Result<()> {
    let name = "python_basic";
    common::run_test(&format!("python {name}"))?;
    common::run_bash_script(name, "python3 main.py")?;
    Ok(())
}
