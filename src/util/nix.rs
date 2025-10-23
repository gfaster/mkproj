use std::fmt::{Display, Write};

use anyhow::{bail, Result};

use crate::util::{self, indentd};

type Str = Box<str>;

/// simple builder for `shell.nix` files
#[derive(Default, Debug)]
pub struct NixBuilder {
    rec: bool,

    name: Option<Str>,

    /// variables used in `let ... in ` structure
    let_vars: Vec<(Str, Str)>,
    /// packages in build inputs - we use vecs here to maintain order
    build_inputs: Vec<Str>,
    /// other attributes
    attrs: Vec<(Str, Str, Option<Str>)>,

    /// value of LD_LIBRARY_PATH, constructed with `lib.makeLibraryPath`. If None, it will be
    /// ommitted
    library_path: Option<Vec<Str>>,
}


const SHIFT: u32 = 2;

impl NixBuilder {
    pub fn new() -> Self {
        NixBuilder::default()
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn rec(&mut self) -> &mut Self {
        self.rec = true;
        self
    }

    fn is_attr_set(&self, var: &str) -> bool {
        (var == "buildInputs")
        || self.attrs.iter().find(|(key, _, _)| &**key == var).is_some()
    }

    pub fn build(&self) -> String {
        let mut ret = String::new();
        let mut indent = 0;

        macro_rules! writeln_indent {
            ($($args:tt)*) => {
                writeln!(ret, "{}", indentd(format_args!($($args)*), SHIFT * indent)).expect("writing to String never fails");
            };
        }

        ret.write_str("{ pkgs ? import <nixpkgs> {} }:\n").unwrap();

        if !self.let_vars.is_empty() {
            indent += 1;
            writeln_indent!("let");
            indent += 1;
            for (var, val) in &self.let_vars {
                writeln_indent!("{var} = {val};");
            }
            indent -= 1;
            writeln_indent!("in");
        }

        writeln_indent!("pkgs.mkShell {rec}{{", rec = if self.rec { "rec " } else { "" });
        indent += 1;

        if let Some(name) = self.name.as_deref() {
            writeln_indent!(r#"name = "{}";"#, name.escape_default());
        }

        writeln_indent!("packages = {:#};", fmt_array(Some("pkgs"), &self.build_inputs));

        if let Some(libraries) = self.library_path.as_deref() {
            writeln_indent!("LD_LIBRARY_PATH = {:#};", fmt_array(Some("pkgs"), libraries));
        }

        for (key, attr, comment) in &self.attrs {
            if let Some(comment) = comment {
                writeln_indent!("# {comment}");
            }
            writeln_indent!("{key} = {attr};");
        }
        indent -= 1;

        writeln_indent!("}}");

        ret
    }

    /// Equivalent to: `util::write_to_file("shell.nix", self.build())`
    pub fn write_to_shell_dot_nix(&self) -> Result<()> {
        util::write_to_file("shell.nix", self.build())
    }

    pub fn add_expr_attribute(&mut self, key: impl Into<Box<str>>, attr: impl Display) -> Result<&mut Self> {
        let key = key.into();
        if self.is_attr_set(&key) {
            bail!("attribute {key} is already set")
        }
        self.attrs.push((key, attr.to_string().into(), None));

        Ok(self)
    }

    pub fn add_expr_attribute_comment(&mut self, key: impl Into<Box<str>>, attr: impl Display, comment: &str) -> Result<&mut Self> {
        let key = key.into();
        if self.is_attr_set(&key) {
            bail!("attribute {key} is already set")
        }
        self.attrs.push((key, attr.to_string().into(), Some(comment.into())));

        Ok(self)
    }

    pub fn add_string_attribute(&mut self, key: impl Into<Box<str>>, attr: impl Display) -> Result<&mut Self> {
        let key = key.into();
        if self.is_attr_set(&key) {
            bail!("attribute {key} is already set")
        }
        self.attrs.push((key, format!("''{attr}''").into(), None));
        Ok(self)
    }

    pub fn add_letvar_expr(&mut self, var: impl Into<Box<str>>, val: impl Display) -> Result<&mut Self> {
        let var = var.into();
        if self.let_vars.iter().find(|(v, _)| v == &var).is_some() {
            bail!("letvar {var} is already set")
        }
        self.let_vars.push((var, val.to_string().into()));
        Ok(self)
    }

    pub fn add_letvar_string(&mut self, var: impl Into<Box<str>>, val: impl Display) -> Result<&mut Self> {
        let var = var.into();
        if self.let_vars.iter().find(|(v, _)| v == &var).is_some() {
            bail!("letvar {var} is already set")
        }
        self.let_vars.push((var, format!("''{val}''").into()));
        Ok(self)
    }

    pub fn add_build_input(&mut self, pkg: impl Display) -> &mut Self {
        let pkg = pkg.to_string().into_boxed_str();
        if self.build_inputs.contains(&pkg) {
            return self
        }
        self.build_inputs.push(pkg);
        self
    }

    pub fn add_build_inputs(&mut self, pkgs: impl IntoIterator<Item = impl Display>) -> &mut Self {
        for pkg in pkgs {
            self.add_build_input(pkg);
        }
        self
    }

    pub fn include_ld_library_path(&mut self) -> &mut Self {
        if self.library_path.is_none() {
            self.library_path = Some(vec![])
        }
        self
    }

    pub fn add_library(&mut self, lib_package: impl Display) -> &mut Self {
        let pkg = lib_package.to_string().into_boxed_str();
        self.include_ld_library_path();
        let v = self.library_path.as_mut().unwrap();
        if v.contains(&pkg) {
            return self
        }
        v.push(pkg);
        self
    }

    pub fn add_libraries(&mut self, lib_packages: impl IntoIterator<Item = impl Display>) -> &mut Self {
        for pkg in lib_packages {
            self.add_library(pkg);
        }
        self
    }
}



fn fmt_array<'a, I, T>(namespace: Option<&'a str>, vals: I) -> impl Display + use<'a, I, T>
where 
    I: IntoIterator<Item = T> + Copy,
    T: Display
{
    util::fmt_fn(move |f| {
        let lf = if f.alternate() { "\n" } else { " " };
        if let Some(ns) = namespace {
            write!(f, "with {ns}; [{lf}").unwrap();
        } else {
            write!(f, "[{lf}").unwrap();
        }

        if f.alternate() {
            for val in vals {
                writeln!(f, "{}", indentd(val, SHIFT)).unwrap();
            }
        } else {
            for val in vals {
                write!(f, "{val} ").unwrap();
            }
        }
        write!(f, "]")?;
        Ok(())
    })
}
