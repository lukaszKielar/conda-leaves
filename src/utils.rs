use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use regex::Regex;

use crate::metadata::Metadata;

pub fn split_and_take_n_elem<T: AsRef<str>>(string: &T, n: usize) -> Option<&str> {
    if string.as_ref().is_empty() {
        return None;
    }
    let v = string.as_ref().split(" ").collect::<Vec<_>>();
    let v = *v.get(n).unwrap();
    Some(v)
}

lazy_static! {
    static ref VERSION_REGEX: Regex =
        Regex::new(r"(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)").unwrap();
}

fn extract_version(text: String) -> Option<String> {
    match VERSION_REGEX.find(text.as_ref()) {
        Some(v) => return Some(v.as_str().to_string()),
        None => return None,
    };
}

fn get_conda_prefix() -> String {
    std::env::var("CONDA_PREFIX").expect("CONDA_PREFIX not set. Probably outside of conda env.")
}

pub fn get_conda_meta_path() -> PathBuf {
    let conda_prefix = get_conda_prefix();
    let conda_meta = Path::new(&conda_prefix).join("conda-meta");
    conda_meta
}

pub fn get_conda_metadata() -> HashMap<String, Metadata> {
    let conda_meta = get_conda_meta_path();

    // TODO improvements: I can know the size of HashMap in advance.
    let mut conda_metadata: HashMap<String, Metadata> = HashMap::new();

    for entry in conda_meta.read_dir().unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() & path.to_str().unwrap().ends_with(".json") {
            let metadata = Metadata::from_json(&path).unwrap();
            conda_metadata.insert(metadata.name.clone(), metadata);
        }
    }
    conda_metadata
}

// TODO this function should take a conda_metadata as an argument
//  it will be much more flexible
// TODO this function should take a reference to a name
pub fn get_dependent_packages(name: String) -> Vec<String> {
    let conda_metadata = get_conda_metadata();

    match conda_metadata.get(&name) {
        Some(_) => (),
        None => panic!("Package '{}' not installed", name),
    }

    let dependent_packages: Vec<String> = conda_metadata
        .values()
        .filter(|m| m.requires_dist.contains(&name))
        .map(|m| m.name.clone())
        .filter(|n| !n.starts_with("python"))
        .collect();
    dependent_packages
}

pub fn get_leaves() -> Vec<String> {
    let conda_metadata = get_conda_metadata();

    let mut leaves: Vec<String> = vec![];

    for (name, m) in conda_metadata.iter() {
        // 0 dependent packages means that the package it the leaf
        if get_dependent_packages(name.to_string()).len() == 0 {
            // add name of the package to main dependencies
            leaves.push(name.to_string());
            // and also its dependencies
            leaves.extend(m.requires_dist.clone())
        }
    }
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
        let text = "version 3.7.3".to_string();
        let output = extract_version(text);
        assert_eq!(output, Some("3.7.3".to_string()));
    }

    #[test]
    fn test_extract_version_empty() {
        let text = "version".to_string();
        let output = extract_version(text);
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
            "pkg1".to_string(),
            Metadata::new("pkg1".to_string(), "0.0.1".to_string(), vec![]),
        );
        expected_conda_metadata.insert(
            "pkg2a".to_string(),
            Metadata::new(
                "pkg2a".to_string(),
                "0.0.1".to_string(),
                vec!["pkg1".to_string()],
            ),
        );
        expected_conda_metadata.insert(
            "pkg2b".to_string(),
            Metadata::new("pkg2b".to_string(), "0.0.1".to_string(), vec![]),
        );
        expected_conda_metadata.insert(
            "pkg2c".to_string(),
            Metadata::new(
                "pkg2c".to_string(),
                "0.0.1".to_string(),
                vec!["pkg2a".to_string()],
            ),
        );
        expected_conda_metadata.insert(
            "pkg3".to_string(),
            Metadata::new(
                "pkg3".to_string(),
                "0.0.1".to_string(),
                vec!["pkg2a".to_string(), "pkg2b".to_string()],
            ),
        );
        // when:
        let conda_metadata = get_conda_metadata();
        // then:
        assert_eq!(conda_metadata, expected_conda_metadata)
    }

    #[test]
    fn test_get_dependent_packages_empty() {
        // given:
        std::env::set_var("CONDA_PREFIX", "./tests/data");
        let expected_dependent_packages: Vec<String> = vec![];
        // when:
        let dependent_packages = get_dependent_packages("pkg3".to_string());
        // then:
        assert_eq!(dependent_packages, expected_dependent_packages)
    }

    #[test]
    fn test_get_dependent_packages_one() {
        // given:
        std::env::set_var("CONDA_PREFIX", "./tests/data");
        let expected_dependent_packages: Vec<String> = vec!["pkg3".to_string()];
        // when:
        let dependent_packages = get_dependent_packages("pkg2b".to_string());
        // then:
        assert_eq!(dependent_packages, expected_dependent_packages)
    }

    #[test]
    fn test_get_dependent_packages_multiple() {
        // given:
        std::env::set_var("CONDA_PREFIX", "./tests/data");
        let mut expected_dependent_packages: Vec<String> =
            vec!["pkg3".to_string(), "pkg2c".to_string()];
        expected_dependent_packages.sort();
        // when:
        let mut dependent_packages = get_dependent_packages("pkg2a".to_string());
        dependent_packages.sort();
        // then:
        assert_eq!(dependent_packages, expected_dependent_packages)
    }

    #[test]
    fn test_get_leaves() {
        // given:
        std::env::set_var("CONDA_PREFIX", "./tests/data");
        let mut expected_leaves: Vec<String> = vec![
            "pkg2a".to_string(),
            "pkg2b".to_string(),
            "pkg2c".to_string(),
            "pkg3".to_string(),
        ];
        expected_leaves.sort();
        // when:
        let leaves = get_leaves();
        // then:
        assert_eq!(leaves, expected_leaves)
    }
}
