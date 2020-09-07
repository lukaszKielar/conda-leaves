use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;

pub struct CondaEnv {
    // TODO name should be automatically taken from CONDA_DEFAULT_ENV variable
    name: String,
    // TODO dependencies should be defined as a Vec<Package>
    dependencies: Vec<String>,
    // TODO pip should be defined as a Vec<Package>
    // TODO pip should be deleted, information about installer is available in Package struct
    pip: Option<Vec<String>>,
}

impl CondaEnv {
    pub fn new(name: String, dependencies: Vec<String>, pip: Option<Vec<String>>) -> Self {
        Self {
            name,
            dependencies,
            pip,
        }
    }

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
        for d in self.dependencies.iter() {
            file.write_all(b"\n")?;
            file.write_all(b"  - ")?;
            file.write_all(d.as_bytes())?;
        }
        file.write_all(b"\n")?;

        if let Some(pip_dependencies) = &self.pip {
            file.write_all(b"  - pip:")?;
            for d in pip_dependencies.iter() {
                file.write_all(b"\n")?;
                file.write_all(b"    - ")?;
                file.write_all(d.as_bytes())?;
            }
            file.write_all(b"\n")?;
        }

        Ok(())
    }
}
