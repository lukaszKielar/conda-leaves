mod environment;
mod metadata;
mod package;
mod utils;

use std::io;

use dotenv::dotenv;

use crate::environment::EnvironmentYml;
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

    println!("----to-environment.yml----");
    let env = EnvironmentYml::new(
        String::from("test"),
        vec![String::from("conda1"), String::from("conda2")],
        Some(vec![String::from("pip1"), String::from("pip2")]),
    );
    env.to_file()?;

    Ok(())
}
