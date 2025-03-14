use anyhow::Result;

mod common;


#[test]
fn basic() -> Result<()> {
    let name = "basic";
    common::run_test(&format!("odin {name} --no-makefile"))?;
    common::run_bash_script(name, "test ! -f Makefile && odin run .")?;
    Ok(())
}

#[test]
fn makefile() -> Result<()> {
    let name = "makefile";
    common::run_test(&format!("odin {name}"))?;
    common::run_bash_script(name, "make run")?;
    Ok(())
}

const GRAPHICS_TEST: &str = r##"
odin build . -strict-style -vet -warnings-as-errors
set +e
timeout 2 make run
RES="$?"
set -e
if [ "$RES" != 124 ] ; then
    echo "did not time out"
    exit 1
fi
"##;

#[test]
fn raylib() -> Result<()> {
    let name = "raylib";
    common::run_test(&format!("odin {name} -g raylib"))?;
    common::run_bash_script(name, GRAPHICS_TEST)?;
    Ok(())
}

#[test]
fn no_graphics_fails() -> Result<()> {
    // I want to make sure the GRAPHICS_TEST script fails due to not timing out
    let name = "raylib";
    common::run_test(&format!("odin {name}"))?;
    common::run_bash_script(name, GRAPHICS_TEST).expect_err("graphics test failed to reject exit success");
    Ok(())
}
