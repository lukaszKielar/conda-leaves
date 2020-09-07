use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;

use crate::metadata::Metadata;
use crate::package::{Installer, Package};

// TODO add unit tests
pub struct CondaEnv {
    name: String,
    installed_packages: Vec<Package>,
}

impl CondaEnv {
    // TODO add information about version
    pub fn to_yml(&self) -> io::Result<()> {
        // create a file in current directory
        // TODO path could be passed as an argument
        let path = env::current_dir()?.join("environment2.yml");
        let mut file = fs::File::create(path)?;

        // write name to the file
        file.write_all(b"name: ")?;
        file.write_all(self.name.as_bytes())?;
        file.write_all(b"\n")?;

        // write dependencies
        file.write_all(b"dependencies:")?;

        // get deps installed by conda
        let conda_deps: Vec<&Package> = self
            .installed_packages
            .iter()
            .filter(|&p| p.installer == Installer::Conda)
            .collect();

        // get deps installed by pip
        let pip_deps: Vec<&Package> = self
            .installed_packages
            .iter()
            .filter(|&p| p.installer == Installer::Pip)
            .collect();

        for &d in conda_deps.iter() {
            let p_string: String = d.to_owned().into();
            file.write_all(b"\n")?;
            file.write_all(b"  - ")?;
            file.write_all(p_string.as_bytes())?;
        }
        file.write_all(b"\n")?;

        if pip_deps.len() > 0 {
            file.write_all(b"  - pip:")?;
            for &d in pip_deps.iter() {
                let p_string: String = d.to_owned().into();
                file.write_all(b"\n")?;
                file.write_all(b"    - ")?;
                file.write_all(p_string.as_bytes())?;
            }
            file.write_all(b"\n")?;
        }

        Ok(())
    }
}

impl From<HashMap<String, Metadata>> for CondaEnv {
    fn from(conda_metadata: HashMap<String, Metadata>) -> Self {
        // get conda name
        let name =
            env::var("CONDA_DEFAULT_ENV").expect("Please make sure you have conda env activated.");
        let installed_packages: Vec<Package> = conda_metadata
            .values()
            .map(|m| {
                let p: Package = m.to_owned().into();
                p
            })
            .collect();

        Self {
            name,
            installed_packages,
        }
    }
}

impl From<Vec<String>> for CondaEnv {
    fn from(leaves: Vec<String>) -> Self {
        // get conda name
        let name =
            env::var("CONDA_DEFAULT_ENV").expect("Please make sure you have conda env activated.");
        let installed_packages: Vec<Package> = leaves
            .iter()
            .map(|name| Metadata::from_name(name).unwrap().into())
            .collect();

        Self {
            name,
            installed_packages,
        }
    }
}
