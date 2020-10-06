mod env;
mod metadata;
mod package;
mod utils;

use std::io;

use dotenv::dotenv;
use structopt::StructOpt;

use crate::env::CondaEnv;
use crate::metadata::Metadata;
use crate::package::{print_package, Package};
use crate::utils::get_dependent_packages;
use crate::utils::get_leaves;

// FIXME better to use enum
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "p", long, conflicts_with = "leaves")]
    package: Option<String>,

    /// Prints top level packages in conda environment
    #[structopt(short = "l", long, conflicts_with = "package")]
    leaves: bool,

    /// Prints packages installed by conda
    #[structopt(long)]
    no_pip: Option<bool>,
}

fn main() -> io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let cli: Cli = Cli::from_args();
    print!("{:?}", cli);

    match cli {
        Cli { package, .. } => match package {
            Some(name) => {
                let p: Package = Metadata::from_name(name)?.into();
                println!();
                print_package(&p);
            }
            None => (),
        },
        // TODO check no_pip value!!
        Cli { leaves, .. } => {
            let leaves = get_leaves();
            for leaf in leaves.iter() {
                println!("{}", leaf)
            }
        }
    }

    // println!("{:?}", get_dependent_packages("pkg1"));
    // println!();

    // println!("----package----");
    // let name = "pkg3";
    // let p: Package = Metadata::from_name(name)?.into();
    // print_package(&p);
    // println!();

    // println!("----leaves----");
    // let leaves = get_leaves();
    // for leaf in leaves.iter() {
    //     println!("{}", leaf)
    // }

    // println!("----leaves-to-environment.yml----");
    // let env: CondaEnv = leaves.into();
    // env.to_yml()?;

    Ok(())
}
