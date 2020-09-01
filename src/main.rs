mod metadata;
mod package;
mod utils;

use std::convert::From;
use std::io;

use dotenv::dotenv;
use env_logger;
use log::debug;

use crate::metadata::Metadata;
use crate::package::print_package;
use crate::package::Package;
use crate::utils::get_site_packages_path;

fn main() -> io::Result<()> {
    dotenv().ok();
    env_logger::init();

    // get site-packages folder for the conda env that is activated
    let site_packages = get_site_packages_path();

    // iterate over entries in site-packages
    // and look for folder that endswith `info`
    // (metadata files are stored there)
    for entry in site_packages.read_dir()? {
        let dir = entry?.path();
        if dir.is_dir() & dir.to_str().unwrap().ends_with("egg-info") {
            debug!("{:?}", dir);
        }
    }

    println!("----package----");
    let name = String::from("dask-core");
    let metadata = Metadata::from_name(name).unwrap();
    let package = Package::from(metadata);
    print_package(&package);

    Ok(())
}
