mod cli;
mod env;
mod metadata;
mod package;
mod utils;

use std::io;

use dotenv::dotenv;

use crate::env::CondaEnv;
use crate::metadata::Metadata;
use crate::package::{print_package, Package};
use crate::utils::get_dependent_packages;
use crate::utils::get_leaves;

fn cli_main() {
    dotenv().ok();

    let matches = cli::build_cli().get_matches();

    match matches.subcommand() {
        _ => unreachable!("The cli parser should prevent reaching here"),
    }
}

fn main() -> io::Result<()> {
    dotenv().ok();
    env_logger::init();

    println!("{:?}", get_dependent_packages("pkg1"));
    println!();

    println!("----package----");
    let name = "pkg3";
    let p: Package = Metadata::from_name(name)?.into();
    print_package(&p);
    println!();

    println!("----leaves----");
    let leaves = get_leaves();
    for leaf in leaves.iter() {
        println!("{}", leaf)
    }

    println!("----leaves-to-environment.yml----");
    let env: CondaEnv = leaves.into();
    env.to_yml()?;

    Ok(())
}
