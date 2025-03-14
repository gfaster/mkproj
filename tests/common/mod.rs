use std::{io::Write, os::unix::prelude::{FromRawFd, PermissionsExt}, path::{Path, PathBuf}, process::Command, sync::{Mutex, MutexGuard}};

use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use mkproj::{util::{cd, mkdir}, Cli};

static DO_STUFF_LOCK: Mutex<()> = Mutex::new(());

fn get_lock() -> MutexGuard<'static, ()> {
    loop {
        match DO_STUFF_LOCK.lock() {
            Ok(l) => break l,
            Err(_) => DO_STUFF_LOCK.clear_poison(),
        }
    }
}

pub fn run_test(args: &str) -> Result<()> {
    run_test_with_fn(args, || Ok(()))
}

pub fn run_test_with_fn<T>(args: &str, f: impl FnOnce() -> Result<T>) -> Result<T> {
    clear_target_tmpdir();
    mkproj::enable_test_mode();
    let lock = get_lock();
    cd(proj_dir())?;
    let exe_name = "mkproj";
    let full_args = format!("{exe_name} {args}");
    let cli = Cli::try_parse_from(full_args.split_whitespace()).context("parsing arguments")?;
    mkproj::run_main(&cli).context("executing mkproj")?;
    let ret = f();
    drop(lock);
    ret
}

fn proj_dir() -> PathBuf {
    let pid = std::process::id();
    let base = Path::new(env!("CARGO_TARGET_TMPDIR"));
    base.join(format!("{pid}"))
}

fn clear_target_tmpdir() {
    static LOCK: Mutex<bool> = Mutex::new(false);
    let mut lock = LOCK.lock().unwrap();
    if !*lock {
        if let Err(e) = std::fs::remove_dir_all(proj_dir()) {
            if e.kind() != std::io::ErrorKind::NotFound {
                panic!("{e}")
            }
        }
        mkdir(proj_dir()).unwrap();
        *lock = true
    }
}

/// like [`run_script`] but automatically includes shebang for bash and `set -e`
pub fn run_bash_script(proj_name: &str, script: &str) -> Result<()> {
    let script = format!("#!/usr/bin/env bash\n\nset -e\n{script}");
    run_script(proj_name, &script).context("failed to run bash script")
}

pub fn run_script(proj_name: &str, script: &str) -> Result<()> {
    ensure!(!proj_name.contains('/'), "name contains /");
    let path = proj_dir();
    let path = path.join(mkproj::util::proj_dir_name(proj_name)?);
    let name = format!("{proj_name}_script\0"); // can't use cstr literal in format!
    let name = std::ffi::CString::from_vec_with_nul(name.into())?;
    let fd = unsafe { libc::memfd_create(name.as_ptr(), 0) };
    if fd == -1 {
        let err = std::io::Error::last_os_error();
        return Err(err).context("memfd_create")
    }
    let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
    let mut permissions = file.metadata().context("metadata")?.permissions();
    permissions.set_mode(0o644);
    file.set_permissions(permissions.clone()).context("set memfd permissions for writing")?;
    file.write_all(script.as_bytes()).context("writing to memfd")?;
    permissions.set_mode(0o544);
    file.set_permissions(permissions).context("set memfd permissions for execute")?;
    let output = Command::new("nix-shell")
        .current_dir(path)
        .arg("--pure")
        .arg("--run")
        .arg(format!("/proc/self/fd/{fd}"))
        .output()
        .context("execute memfd script")?;

    if !output.stderr.is_empty() {
        eprintln!("==== nix-shell script stderr ===");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
    if !output.stdout.is_empty() {
        eprintln!("==== nix-shell script stdout ===");
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
    }

    if !output.status.success() {
        bail!("script failed: {}", output.status)
    }

    Ok(())
}
