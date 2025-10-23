use anyhow::Result;
use clap::Args;

use crate::util::{mk_proj_dir, nix::NixBuilder};

#[derive(Args)]
pub(crate) struct CommonArgs {
    /// project name
    #[arg(required = true)]
    pub name: Box<str>,

    #[arg(short, long)]
    /// packages in nixpkgs
    pub package: Vec<Box<str>>
}

impl CommonArgs {
    /// calls [`mk_proj_dir`], sets the name, and adds the manually specified packages
    pub fn begin(&self) -> Result<NixBuilder> {
        mk_proj_dir(&self.name)?;
        let mut nix = crate::util::nix::NixBuilder::new();
        nix.set_name(&self.name);
        nix.add_build_inputs(&self.package);

        Ok(nix)
    }
}
