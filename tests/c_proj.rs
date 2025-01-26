mod common;


#[test]
fn basic_make_simple() -> anyhow::Result<()> {
    let name = "make_simple";
    common::run_test(&format!("c {name} -b make-simple"))?;
    common::run_bash_script(name, "make all")?;
    Ok(())
}

#[test]
fn basic_make_normal() -> anyhow::Result<()> {
    let name = "basic_make_normal";
    common::run_test(&format!("c {name} -b make"))?;
    common::run_bash_script(name, &format!("make all; build/{name}"))?;
    Ok(())
}

#[test]
fn basic_cmake() -> anyhow::Result<()> {
    let name = "cmake_basic";
    let script = format!(r##"
    cd build
    cmake ..
    make
    ./{name}
    "##);
    common::run_test(&format!("c {name} -b cmake"))?;
    common::run_bash_script(name, &script)?;
    Ok(())
}

#[test]
fn make_cxx() -> anyhow::Result<()> {
    let name = "make_cxx";
    common::run_test(&format!("c {name} -b make --force-cxx"))?;
    common::run_bash_script(name, &format!("make all; build/{name}"))?;
    Ok(())
}

#[test]
fn basic_cmake_cxx() -> anyhow::Result<()> {
    let name = "basic_cmake_cxx";
    let script = format!(r##"
    cd build
    cmake ..
    make
    ./{name}
    "##);
    common::run_test(&format!("c {name} -b cmake --force-cxx"))?;
    common::run_bash_script(name, &script)?;
    Ok(())
}
