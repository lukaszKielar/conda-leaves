mod env;
mod metadata;
mod package;
mod utils;

use std::io;
use std::path::PathBuf;

use dotenv::dotenv;
use structopt::StructOpt;

use crate::env::CondaEnv;
use crate::metadata::Metadata;
use crate::package::{print_package, Package};
use crate::utils::get_dependent_packages;
use crate::utils::get_leaves;

#[derive(Debug, StructOpt)]
#[structopt(name = "conda-leaves")]
enum Opts {
    /// Prints top level packages in conda environment
    #[structopt(name = "leaves")]
    Leaves {
        /// Prints packages installed by conda only
        #[structopt(long)]
        no_pip: bool,
    },

    /// Prints tree view for the package
    #[structopt(name = "package")]
    Package {
        #[structopt(short = "n", long)]
        name: String,
    },
    /// Exports leaves to the file
    #[structopt(name = "export")]
    Export {
        #[structopt(short = "f", long, default_value = "env.yml", parse(from_os_str))]
        filename: PathBuf,
    },
}

fn main() -> io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let opts: Opts = Opts::from_args();
    println!("{:?}", opts);

    match opts {
        Opts::Package { name } => {
            let p: Package = Metadata::from_name(name)?.into();
            println!();
            print_package(&p);
        }
        Opts::Leaves { no_pip } => match no_pip {
            false => {
                let leaves = get_leaves();
                println!();
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
        _ => (),
    }

    Ok(())
}
