mod metadata;
mod package;
mod utils;

use std::io;

use dotenv::dotenv;

use crate::metadata::Metadata;
use crate::package::{print_package, Package};
use crate::utils::get_dependent_packages;
use crate::utils::get_leaves;

fn main() -> io::Result<()> {
    dotenv().ok();
    env_logger::init();

    println!("{:?}", get_dependent_packages("pkg1"));
    println!();

    println!("----package----");
    let name = "pkg3";
    let p: Package = Metadata::from_name(name).unwrap().into();
    print_package(&p);
    println!();

    println!("----leaves----");
    for leaf in get_leaves().iter() {
        println!("{}", leaf)
    }

    Ok(())
}
