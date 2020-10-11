mod env;
mod metadata;
mod package;
mod utils;

use std::io;
use std::path::PathBuf;

use structopt::StructOpt;

use crate::env::CondaEnv;
use crate::metadata::Metadata;
use crate::package::{print_package, Package};
use crate::utils::get_leaves;

#[derive(Debug, StructOpt)]
#[structopt(name = "conda-leaves")]
enum Opts {
    /// Prints top level packages in conda environment
    Leaves {
        /// Prints packages installed by conda only
        #[structopt(long)]
        no_pip: bool,
    },
    /// Prints tree view for the package
    Package {
        #[structopt(short = "n", long)]
        name: String,
    },
    /// Exports leaves to the file
    Export {
        #[structopt(
            short = "f",
            long,
            default_value = "environment.yml",
            parse(from_os_str)
        )]
        filename: PathBuf,
    },
}

fn main() -> io::Result<()> {
    let opts: Opts = Opts::from_args();

    match opts {
        Opts::Package { name } => match Metadata::from_name(name) {
            Ok(m) => {
                let p: Package = m.into();
                print_package(&p);
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1)
            }
        },
        Opts::Leaves { no_pip } => match no_pip {
            false => {
                let leaves = get_leaves();
                for leaf in leaves.iter() {
                    println!("{}", leaf)
                }
            }
            // FIXME for now we support conda only
            true => println!("Mixed conda-pip environments are not supported yet."),
        },
        Opts::Export { filename } => {
            let leaves = get_leaves();
            let env: CondaEnv = leaves.into();
            env.to_yml(&filename)?
        }
    }

    Ok(())
}
