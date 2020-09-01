use std::collections::HashMap;
use std::path::{Path, PathBuf};

use glob::glob;
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

pub fn get_site_packages_path() -> PathBuf {
    let conda_prefix = get_conda_prefix();
    let mut site_packages = String::new();

    for entry in glob(&conda_prefix).unwrap() {
        match entry {
            Ok(path) => {
                site_packages.push_str(path.to_str().unwrap());
                break;
            }
            _ => (),
        }
    }

    Path::new(&site_packages).into()
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
}
