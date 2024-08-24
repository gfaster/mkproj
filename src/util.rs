#![allow(dead_code)]

pub mod nix;

use std::{fmt, fs, io, path::Path};

use anyhow::{bail, ensure, Context, Result};

struct Indent<D>(D, u32);
impl<D: fmt::Display> fmt::Display for Indent<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0.to_string();
        for line in s.split_inclusive('\n') {
            for _ in 0..self.1 {
                write!(f, " ")?;
            }
            write!(f, "{line}")?;
        }
        Ok(())
    }
}
pub fn indentd(d: impl fmt::Display, amt: u32) -> impl fmt::Display {
    Indent(d, amt)
}

pub fn indent(amt: u32) -> impl fmt::Display {
    Indent("", amt)
}

pub fn tr_map(d: impl fmt::Display, from: char, to: char) -> impl fmt::Display {
    let f = move |c| if c == from { to } else { c };
    tr(d, f)
}

pub fn tr(d: impl fmt::Display, f: impl Fn(char) -> char) -> impl fmt::Display {
    use std::fmt::Write;
    struct P<D, F>(D, F);
    impl<D: fmt::Display, F: Fn(char) -> char> fmt::Display for P<D, F> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for c in self.0.to_string().chars() {
                f.write_char((self.1)(c))?
            }
            Ok(())
        }
    }
    P(d, f)
}

/// create a new, empty file, failing if it exists
pub fn touch_new(path: impl AsRef<Path>) -> Result<()> {
    let p = path.as_ref();
    fs::OpenOptions::new().create_new(true).write(true).open(p).with_context(file_error("failed to create", p))?;
    print_path_op("create", p);
    Ok(())
}

pub fn write_to_file(path: impl AsRef<Path>, content: impl fmt::Display) -> Result<()> {
    use std::io::Write;
    let p = path.as_ref();
    let file = fs::OpenOptions::new().create_new(true).write(true).open(p).with_context(file_error("failed to create", p))?;
    let mut file = std::io::BufWriter::new(file);
    write!(file, "{content}").with_context(file_error("failed to write to", p))?;
    file.flush().context("flush failed")?;
    print_path_op("create", p);
    Ok(())
}

pub fn git_init() -> Result<()> {
    use std::process::{Command, Stdio};

    let out = Command::new("git").arg("init").stdout(Stdio::inherit()).stderr(Stdio::inherit()).output().context("failed to run git init")?;
    ensure!(out.status.success(), "git init failed with code {}", out.status);
    Ok(())
}

/// does common actions to create empty project directory
///
/// - verifies project name is OK
/// - fails if directory already exists
/// - changes current directory to project directory
pub fn mk_proj_dir(proj: &str) -> Result<()> {

    let path: &Path = "/home/".as_ref();
    let mut path = path.to_owned();
    if let Some(home) = std::env::var_os("HOME") {
        path.push(home)
    } else {
        bail!("$HOME is not set")
    }
    path.push("projects");
    if !path.try_exists().context("could not determine if project dir exists")? {
        bail!("projects dir '{}' doesn't exist - make it first", path.display());
    }
    path.push(proj);
    if path.try_exists().context("could not determine if path exists")? {
        bail!("project path '{}' already exists", path.display());
    }
    // don't use mkdir_all since we want to create exactly one new dir
    mkdir(&path)?;
    cd(&path)?;
    Ok(())
}

pub fn cd(p: impl AsRef<Path>) -> Result<()> {
    let p = p.as_ref();
    std::env::set_current_dir(&p).with_context(file_error("failed to cd to", p))?;
    print_path_op("cd", p);
    Ok(())
}

pub fn mkdir(p: impl AsRef<Path>) -> Result<()> {
    fn inner(p: &Path) -> Result<()> {
        fs::create_dir(p).with_context(file_error("failed to create dir", p))?;
        print_path_op("mkdir", p);
        Ok(())
    }
    inner(p.as_ref())
}

pub fn mkdir_all(p: impl AsRef<Path>) -> Result<()> {
    let p = p.as_ref();
    // based off std impl
    // https://doc.rust-lang.org/src/std/fs.rs.html#2690
    fn inner(path: &Path) -> Result<()> {
        if path == Path::new("") {
            return Ok(());
        }
        match fs::create_dir(path) {
            Ok(()) => {
                print_path_op("mkdir", path);
                return Ok(())
            },
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(_) if path.is_dir() => {
                print_path_op("mkdir", path);
                return Ok(())
            },
            Err(e) => return Err(e.into()),
        }
        match path.parent() {
            Some(p) => inner(p)?,
            None => {
                bail!("{}", file_error("failed to create whole tree at", path)())
            }
        }
        match fs::create_dir(path) {
            Ok(()) => {
                print_path_op("mkdir", path);
                Ok(())
            },
            Err(_) if path.is_dir() => {
                print_path_op("mkdir", path);
                Ok(())
            },
            Err(e) => Err(e.into()),
        }
    }
    inner(p)
}

fn print_path_op(op: &str, p: &Path) {
    eprintln!("{op}: '{}'", p.display())
}

const fn file_error<'a>(msg: &'static str, p: &'a Path) -> impl FnOnce() -> String + 'a {
    move || format!("{msg} '{}'", p.display())
}
