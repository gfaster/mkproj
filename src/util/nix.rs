use std::fmt::{Display, Write};

use anyhow::{bail, Result};

use crate::util::indentd;

type Str = Box<str>;

/// simple builder for `shell.nix` files
#[derive(Default, Debug)]
pub struct NixBuilder {
    rec: bool,
    /// variables used in `let ... in ` structure
    let_vars: Vec<(Str, Str)>,
    /// packages in build inputs - we use vecs here to maintain order
    build_inputs: Vec<Str>,
    /// other attributes
    attrs: Vec<(Str, Str, Option<Str>)>,
}


const SHIFT: u32 = 2;

impl NixBuilder {
    pub fn new() -> Self {
        NixBuilder::default()
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

        ret.write_str("{ pkgs ? import <nixpkgs> {} }:\n").unwrap();

        if !self.let_vars.is_empty() {
            writeln!(ret, "{}", indentd("let", SHIFT)).unwrap();
            for (var, val) in &self.let_vars {
                writeln!(ret, "{}", indentd(format_args!("{var} = {val};"), SHIFT * 2)).unwrap();
            }
            writeln!(ret, "{}", indentd("in", SHIFT)).unwrap();
        }

        write!(ret, "{}", indentd("pkgs.mkShell ", SHIFT)).unwrap();
        if self.rec {
            ret.push_str("rec ")
        }
        ret.push_str("{\n");

        writeln!(ret, "{}", indentd(format_args!("buildInputs = {};", fmt_array(Some("pkgs"), true, &self.build_inputs)), SHIFT * 2)).unwrap();

        for (key, attr, comment) in &self.attrs {
            if let Some(comment) = comment {
                writeln!(ret, "{}", indentd(format_args!("# {comment}"), SHIFT * 2)).unwrap();
            }
            writeln!(ret, "{}", indentd(format_args!("{key} = {attr};"), SHIFT * 2)).unwrap();
        }

        writeln!(ret, "{}", indentd('}', SHIFT)).unwrap();

        ret
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
        if self.build_inputs.iter().find(|&x| x == &pkg).is_some() {
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
}



fn fmt_array(namespace: Option<&str>, multiline: bool, vals: impl IntoIterator<Item = impl std::fmt::Display>) -> impl Display {
    let mut buf = String::new();
    let lf = if multiline { "\n" } else { " " };
    if let Some(ns) = namespace {
        write!(buf, "with {ns}; [{lf}").unwrap();
    } else {
        write!(buf, "[{lf}").unwrap();
    }

    if multiline {
        for val in vals {
            writeln!(buf, "{}", indentd(val, SHIFT)).unwrap();
        }
    } else {
        for val in vals {
            write!(buf, "{val} ").unwrap();
        }
    }
    write!(buf, "]").unwrap();
    buf
}
