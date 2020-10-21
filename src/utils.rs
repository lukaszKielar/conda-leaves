use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;

use crate::metadata::Metadata;

#[doc(hidden)]
pub(crate) fn split_and_take_n_elem<T: AsRef<str>>(string: &T, n: usize) -> Option<&str> {
    if string.as_ref().is_empty() {
        return None;
    }
    let v = string.as_ref().split(" ").collect::<Vec<_>>();
    let v = *v.get(n).unwrap();
    Some(v)
}

lazy_static! {
    #[doc(hidden)]
    static ref VERSION_REGEX: Regex =
        Regex::new(r"(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)").unwrap();
}

lazy_static! {
    #[doc(hidden)]
    pub(crate) static ref CONDA_METADATA: HashMap<String, Metadata> = get_conda_metadata();
}

#[doc(hidden)]
/// Returns optional version (as a String) from given text, when it gets a match again version regex.
fn extract_version<T: AsRef<str>>(text: T) -> Option<String> {
    match VERSION_REGEX.find(text.as_ref()) {
        Some(v) => return Some(v.as_str().to_string()),
        None => return None,
    };
}

#[doc(hidden)]
/// Returns CONDA_PREFIX evironment variable.
///
/// Panics if CONDA_PREFIX is not specified.
fn get_conda_prefix() -> String {
    match std::env::var("CONDA_PREFIX") {
        Ok(var) => return var,
        Err(e) => panic!(e.to_string()),
    }
}

/// Returns `conda-meta` path for activated conda environment.
pub(crate) fn get_conda_meta_path() -> PathBuf {
    let conda_prefix = get_conda_prefix();
    let conda_meta = Path::new(&conda_prefix).join("conda-meta");
    conda_meta
}

/// Returns the dictionary of all installed Python packages with environment, by reading all available metadata files.
///
/// It returns the HashMap, where:
/// - `key` - is the name of the package.
/// - `value` - is the Metadata object instance, that contains some additional information about the package.
///
/// It's main `conda-leaves` data structure, that contains all the details about currently activated conda environment.
/// Output of the function is assigned to `CONDA_METADATA` static variable using `lazy_static!`.
/// It's been designed like that to avoid multiple IO operations, the result is been generated once, and reused.
pub(crate) fn get_conda_metadata() -> HashMap<String, Metadata> {
    let conda_meta = get_conda_meta_path();

    // read conda meta directory and get all of the json metadata files
    let json_metadata_files: Vec<_> = conda_meta
        .read_dir()
        .unwrap()
        .map(|direntry| direntry.unwrap().path())
        .filter(|path| path.is_file() & path.to_str().unwrap().ends_with(".json"))
        .collect();

    // iterate over json files and create hashmap of all packages installed
    let conda_metadata: HashMap<String, Metadata> = json_metadata_files
        .par_iter()
        .map(|path| {
            let metadata = Metadata::from_json(&path).unwrap();
            (metadata.name.clone(), metadata)
        })
        .collect();

    conda_metadata
}

/// Returns a list of dependencies for given package.
///
/// Panics if given package name is not present in the environment.
// TODO this should return Result<Vec<String>>
pub fn get_dependent_packages<T: AsRef<str>>(name: T) -> Vec<String> {
    match CONDA_METADATA.get(name.as_ref()) {
        Some(_) => (),
        None => panic!("Package '{}' not installed", name.as_ref()),
    }

    let dependent_packages: Vec<String> = CONDA_METADATA
        .values()
        .filter(|m| m.requires_dist.contains(&name.as_ref().to_owned()))
        .map(|m| m.name.clone())
        .filter(|n| !n.starts_with("python"))
        .collect();
    dependent_packages
}

/// Returns a list of packages that are not defined as a dependency for any other package in the environment.
pub fn get_leaves() -> Vec<String> {
    // filtering
    // 1. packages that are not dependend on any other packages
    // skipping
    // 1. packages that starts with `lib`
    // 2. packages that starts with `_` (underscore), they are really low level
    let mut leaves: Vec<String> = CONDA_METADATA
        .keys()
        .filter(|name| get_dependent_packages(name).len() == 0)
        .filter(|name| !(name.starts_with("lib") || name.starts_with("_")))
        .map(|name| name.to_string())
        .collect();
    // sort vector
    leaves.sort();
    // remove duplicated values
    leaves.dedup();
    // and return them
    leaves
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_and_take_n_elem() {
        let input = "some_pkg (v.1.1.1)";
        assert_eq!(split_and_take_n_elem(&input, 0), Some("some_pkg"));
        assert_eq!(split_and_take_n_elem(&input, 1), Some("(v.1.1.1)"));
    }

    #[test]
    fn test_split_and_take_n_elem_empty_string() {
        let input = "";
        assert_eq!(split_and_take_n_elem(&input, 0), None);
        assert_eq!(split_and_take_n_elem(&input, 1), None);
    }

    #[test]
    fn test_extract_version() {
        let output = extract_version("version 3.7.3");
        assert_eq!(output, Some("3.7.3".to_string()));
    }

    #[test]
    fn test_extract_version_empty() {
        let output = extract_version("version");
        assert_eq!(output, None);
    }

    #[test]
    fn test_get_conda_prefix() {
        // given:
        std::env::set_var("CONDA_PREFIX", "./tests/data");
        let expected_conda_prefix = String::from("./tests/data");
        // when:
        let conda_prefix = get_conda_prefix();
        // then:
        assert_eq!(conda_prefix, expected_conda_prefix)
    }

    #[test]
    #[should_panic(expected = "environment variable not found")]
    fn test_get_conda_prefix_panic() {
        // given:
        std::env::remove_var("CONDA_PREFIX");
        // when:
        get_conda_prefix();
    }

    #[test]
    fn test_get_conda_meta_path() {
        // given:
        std::env::set_var("CONDA_PREFIX", "./tests/data");
        let expected_conda_meta_path = String::from("./tests/data/conda-meta");
        // when:
        let conda_meta_path = get_conda_meta_path().to_str().unwrap().to_string();
        // then:
        assert_eq!(conda_meta_path, expected_conda_meta_path)
    }

    #[test]
    fn test_get_conda_metadata() {
        // given:
        std::env::set_var("CONDA_PREFIX", "./tests/data");
        let mut expected_conda_metadata: HashMap<String, Metadata> = HashMap::new();
        expected_conda_metadata.insert(
            String::from("pkg1"),
            Metadata {
                name: String::from("pkg1"),
                version: String::from("0.0.1"),
                requires_dist: vec![],
            },
        );
        expected_conda_metadata.insert(
            String::from("pkg2a"),
            Metadata {
                name: String::from("pkg2a"),
                version: String::from("0.0.1"),
                requires_dist: vec![String::from("pkg1")],
            },
        );
        expected_conda_metadata.insert(
            String::from("pkg2b"),
            Metadata {
                name: String::from("pkg2b"),
                version: String::from("0.0.1"),
                requires_dist: vec![],
            },
        );
        expected_conda_metadata.insert(
            String::from("pkg2c"),
            Metadata {
                name: String::from("pkg2c"),
                version: String::from("0.0.1"),
                requires_dist: vec![String::from("pkg2a")],
            },
        );
        expected_conda_metadata.insert(
            String::from("pkg3"),
            Metadata {
                name: String::from("pkg3"),
                version: String::from("0.0.1"),
                requires_dist: vec![String::from("pkg2a"), String::from("pkg2b")],
            },
        );
        // when:
        let conda_metadata = get_conda_metadata();
        // then:
        assert_eq!(conda_metadata, expected_conda_metadata)
    }

    #[test]
    #[should_panic(expected = "Package 'pkg404' not installed")]
    fn test_get_dependent_packages_invalid_package_panic() {
        // given:
        std::env::set_var("CONDA_PREFIX", "./tests/data");
        // then:
        get_dependent_packages(String::from("pkg404"));
    }

    #[test]
    fn test_get_dependent_packages_empty() {
        // given:
        std::env::set_var("CONDA_PREFIX", "./tests/data");
        let expected_dependent_packages: Vec<String> = vec![];
        // when:
        let dependent_packages = get_dependent_packages(String::from("pkg3"));
        // then:
        assert_eq!(dependent_packages, expected_dependent_packages)
    }

    #[test]
    fn test_get_dependent_packages_one() {
        // given:
        std::env::set_var("CONDA_PREFIX", "./tests/data");
        let expected_dependent_packages = vec![String::from("pkg3")];
        // when:
        let dependent_packages = get_dependent_packages(String::from("pkg2b"));
        // then:
        assert_eq!(dependent_packages, expected_dependent_packages)
    }

    #[test]
    fn test_get_dependent_packages_multiple() {
        // given:
        std::env::set_var("CONDA_PREFIX", "./tests/data");
        let mut expected_dependent_packages = vec![String::from("pkg3"), String::from("pkg2c")];
        expected_dependent_packages.sort();
        // when:
        let mut dependent_packages = get_dependent_packages(String::from("pkg2a"));
        dependent_packages.sort();
        // then:
        assert_eq!(dependent_packages, expected_dependent_packages)
    }

    #[test]
    fn test_get_leaves() {
        // given:
        std::env::set_var("CONDA_PREFIX", "./tests/data");
        let mut expected_leaves = vec![String::from("pkg2c"), String::from("pkg3")];
        expected_leaves.sort();
        // when:
        let leaves = get_leaves();
        // then:
        assert_eq!(leaves, expected_leaves)
    }
}
