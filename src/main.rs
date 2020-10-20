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

/// Simple CLI tool that allows to pretty print all dependencies within conda environment
#[derive(Debug, StructOpt)]
#[structopt(name = "conda-leaves")]
struct Opts {
    /// Prints packages installed by conda only
    #[structopt(long)]
    no_pip: bool,

    #[structopt(subcommand)]
    commands: Option<Commands>,
}

#[derive(Debug, StructOpt)]
enum Commands {
    /// Prints tree view for the package
    Package {
        #[structopt(short = "n", long)]
        name: String,
    },
    /// Exports leaves to the file
    Export {
        /// Name of the output yml file
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
        Opts {
            no_pip: false,
            commands,
        } => match commands {
            None => {
                let leaves = get_leaves();
                for leaf in leaves.iter() {
                    println!("{}", leaf)
                }
            }
            Some(command) => match command {
                Commands::Package { name } => match Metadata::from_name(name) {
                    Ok(m) => {
                        let p: Package = m.into();
                        print_package(&p);
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(404)
                    }
                },
                Commands::Export { filename } => {
                    let leaves = get_leaves();
                    let env: CondaEnv = leaves.into();
                    env.to_yml(&filename)?
                }
            },
        },
        Opts { no_pip: true, .. } => {
            eprintln!("Mixed conda-pip environments are not supported yet.");
            std::process::exit(1)
        }
    }

    Ok(())
}
