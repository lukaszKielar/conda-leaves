mod metadata;
mod package;
mod utils;

use std::convert::From;
use std::io;

use dotenv::dotenv;

use crate::metadata::Metadata;
use crate::package::{print_package, Package};
use crate::utils::get_conda_metadata;
use crate::utils::get_dependent_packages;

fn main() -> io::Result<()> {
    dotenv().ok();
    env_logger::init();

    // let conda_metadata = get_conda_metadata();
    // for entry in conda_metadata.keys() {
    //     println!("{:?}", conda_metadata.get(entry).unwrap());
    // }
    println!("{:?}", get_dependent_packages("pkg1".to_string()));

    println!("----package----");
    let name = String::from("pkg3");
    let metadata = Metadata::from_name(name).unwrap();
    let package = Package::from(metadata);
    print_package(&package);

    Ok(())
}
