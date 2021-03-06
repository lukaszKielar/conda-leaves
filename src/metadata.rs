use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, ErrorKind};
use std::marker::PhantomData;
use std::path::Path;

use serde::de;
use serde::{Deserialize, Deserializer};

use crate::utils::{split_and_take_n_elem, CONDA_METADATA};

// TODO I may want to consider adding `metadata_version` field
// I assume compatibility with PEP 566 - Metadata v2.1
// https://www.python.org/dev/peps/pep-0566/
/// Core primitive of the library that represents Python package metadata.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub version: String,
    #[serde(
        rename(deserialize = "depends"),
        deserialize_with = "string_or_seq_string"
    )]
    pub requires_dist: Vec<String>,
}

impl Metadata {
    /// Returns new instance of Metadata from given Metadata file path.    
    /// It's able to parse files such as PKG-INFO and METADATA.
    ///
    /// If an invalid path is given, then an error is returned.
    pub fn from_metadata_file<'a, P>(path: &'a P) -> Result<Self, io::Error>
    where
        P: 'a + ?Sized + AsRef<Path>,
    {
        let mut name: String = String::new();
        let mut version: String = String::new();
        let mut requires_dist: Vec<String> = vec![];

        let input = fs::File::open(path)?;
        let buffered = BufReader::new(input);

        for line in buffered.lines().map(|l| l.unwrap()) {
            if line.starts_with("Name") {
                let package_name = split_and_take_n_elem(&line, 1).unwrap();
                name.push_str(package_name);
            } else if line.starts_with("Version") {
                let package_version = split_and_take_n_elem(&line, 1).unwrap();
                version.push_str(package_version);
            } else if line.starts_with("Provides-Extra") {
                // I care only about default requirements,
                // so if I find this line I quit
                break;
            } else if line.starts_with("Requires-Dist") {
                let dependency_name = split_and_take_n_elem(&line, 1).unwrap();
                requires_dist.push(dependency_name.to_string())
            }
        }

        let metadata = Self {
            name,
            version,
            requires_dist,
        };
        Ok(metadata)
    }

    /// Returns new instance of Metadata from given json metadata file path
    /// (conda keeps json metadata files in `conda-meta` folder within environment).
    ///
    /// If an invalid path is given, then an error is returned.
    pub fn from_json<'a, P>(path: &P) -> Result<Self, io::Error>
    where
        P: 'a + ?Sized + AsRef<Path>,
    {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let m = serde_json::from_reader(reader)?;
        Ok(m)
    }

    // TODO for now it supports only conda environments where all packages were installed by conda
    //  mixed (pip-conda) environments will be supported soon
    /// Returns new instance of Metadata from given name.
    /// It goes through `CONDA_METADATA` and tries to get package from given name.
    ///
    /// If an invalid name is given, then an error is returned.
    pub fn from_name<T: AsRef<str>>(name: T) -> Result<Self, io::Error> {
        match CONDA_METADATA.get(name.as_ref()) {
            Some(metadata) => {
                let m = metadata.clone();
                return Ok(m);
            }
            None => {
                return Err(io::Error::new(
                    ErrorKind::Other,
                    format!("Package '{}' not found", name.as_ref()),
                ))
            }
        }
    }
}

#[doc(hidden)]
/// Function is used for custom serialization of the `required_dist` Metadata attribute.
/// It's able to skip some low level dependencies during deserialization.
/// It looks for `depends` section in Metadata files.
fn string_or_seq_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec(PhantomData<Vec<String>>);

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let depends = value.to_owned();
            // skipping
            // 1. packages that starts with `python`
            // 2. packages that starts with `lib`
            // 3. packages that starts with `_` (underscore), they are really low level
            if depends.starts_with("python")
                || depends.starts_with("lib")
                || depends.starts_with("_")
            {
                return Ok(vec![]);
            } else {
                return Ok(vec![depends]);
            }
        }

        fn visit_seq<S>(self, mut visitor: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            let mut new_seq: Vec<String> = vec![];
            while let Some(item) = visitor.next_element::<String>()? {
                let seq = item.split(" ").collect::<Vec<_>>();
                let new_item = seq.get(0).unwrap().to_string();
                // skipping
                // 1. packages that starts with `python`
                // 2. packages that starts with `lib`
                // 3. packages that starts with `_` (underscore), they are really low level
                if new_item.starts_with("python")
                    || new_item.starts_with("lib")
                    || new_item.starts_with("_")
                {
                    continue;
                } else {
                    new_seq.push(new_item)
                }
            }
            Ok(new_seq)
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_from_file_metadata_requires_dist_empty() {
        let path = "./tests/data/site-packages/numpy-1.19.1.dist-info/METADATA";
        let metadata = Metadata::from_metadata_file(path).unwrap();
        assert_eq!(
            metadata,
            Metadata {
                name: String::from("numpy"),
                version: String::from("1.19.1"),
                requires_dist: vec![]
            }
        )
    }

    #[test]
    fn test_from_file_metadata_requires_dist_non_empty() {
        let path = "./tests/data/site-packages/astroid-2.4.2.dist-info/METADATA";
        let metadata = Metadata::from_metadata_file(path).unwrap();
        assert_eq!(
            metadata,
            Metadata {
                name: String::from("astroid"),
                version: String::from("2.4.2"),
                requires_dist: vec![
                    String::from("lazy-object-proxy"),
                    String::from("six"),
                    String::from("wrapt"),
                    String::from("typed-ast"),
                ]
            }
        )
    }

    #[test]
    fn test_from_file_metadata_requires_dist_non_empty_provides_extra_non_empty() {
        let path = "./tests/data/site-packages/mypy-0.782.dist-info/METADATA";
        let metadata = Metadata::from_metadata_file(path).unwrap();
        assert_eq!(
            metadata,
            Metadata {
                name: String::from("mypy"),
                version: String::from("0.782"),
                requires_dist: vec![
                    String::from("typed-ast"),
                    String::from("typing-extensions"),
                    String::from("mypy-extensions"),
                ]
            }
        )
    }

    #[test]
    fn test_from_file_pkginfo_requires_dist_empty() {
        let path = "./tests/data/site-packages/certifi-2020.6.20-py3.8.egg-info/PKG-INFO";
        let metadata = Metadata::from_metadata_file(path).unwrap();
        assert_eq!(
            metadata,
            Metadata {
                name: String::from("certifi"),
                version: String::from("2020.6.20"),
                requires_dist: vec![]
            }
        )
    }

    #[test]
    fn test_from_file_pkginfo_requires_dist_non_empty() {
        let path = "./tests/data/site-packages/pkg1-1.0.0-just-PKG-INFO.egg-info/PKG-INFO";
        let metadata = Metadata::from_metadata_file(path).unwrap();
        assert_eq!(
            metadata,
            Metadata {
                name: String::from("pkg1"),
                version: String::from("1.0.0"),
                requires_dist: vec![String::from("pkg2")]
            }
        )
    }

    #[test]
    fn test_from_json_no_dependencies() {
        // given:
        let path = "./tests/data/conda-meta/pkg1-0.0.1.json";
        let expected_m = Metadata {
            name: String::from("pkg1"),
            version: String::from("0.0.1"),
            requires_dist: vec![],
        };
        // when:
        let m = Metadata::from_json(path).unwrap();
        // then:
        assert_eq!(m, expected_m)
    }

    #[test]
    fn test_from_json_one_dependency() {
        // given:
        let path = "./tests/data/conda-meta/pkg2a-0.0.1.json";
        let expected_m = Metadata {
            name: String::from("pkg2a"),
            version: String::from("0.0.1"),
            requires_dist: vec![String::from("pkg1")],
        };
        // when:
        let m = Metadata::from_json(path).unwrap();
        // then:
        assert_eq!(m, expected_m)
    }

    #[test]
    fn test_from_json_multiple_dependencies() {
        // given:
        let path = "./tests/data/conda-meta/pkg3-0.0.1.json";
        let expected_m = Metadata {
            name: String::from("pkg3"),
            version: String::from("0.0.1"),
            requires_dist: vec![String::from("pkg2a"), String::from("pkg2b")],
        };
        // when:
        let m = Metadata::from_json(path).unwrap();
        // then:
        assert_eq!(m, expected_m)
    }

    #[test]
    fn test_from_name() {
        // given:
        std::env::set_var("CONDA_PREFIX", "./tests/data");
        let expected_name = String::from("pkg3");
        let expected_requires_dist = vec![String::from("pkg2a"), String::from("pkg2b")];
        // when:
        let m = Metadata::from_name(String::from("pkg3")).unwrap();
        // then:
        assert_eq!(m.name, expected_name);
        assert_eq!(m.requires_dist, expected_requires_dist)
    }

    #[test]
    fn test_from_name_unknown_package() {
        // given:
        std::env::set_var("CONDA_PREFIX", "./tests/data");
        // when:
        let m = Metadata::from_name(String::from("unknown"));
        // then:
        assert_eq!(m.is_err(), true)
    }

    #[test]
    fn test_deserialize_metadata_single_depends() {
        // given:
        let string = r#"{
            "name": "pkg1",
            "version": "0.0.1",
            "depends": "pkg2"
        }"#;
        let expected_m = Metadata {
            name: String::from("pkg1"),
            version: String::from("0.0.1"),
            requires_dist: vec![String::from("pkg2")],
        };
        // when:
        let m: Metadata = serde_json::from_str(string).unwrap();
        // then:
        assert_eq!(m, expected_m)
    }

    #[test]
    fn test_deserialize_metadata_skip_python() {
        // given:
        let string = r#"{
            "name": "pkg1",
            "version": "0.0.1",
            "depends": "python"
        }"#;
        let expected_m = Metadata {
            name: String::from("pkg1"),
            version: String::from("0.0.1"),
            requires_dist: vec![],
        };
        // when:
        let m: Metadata = serde_json::from_str(string).unwrap();
        // then:
        assert_eq!(m, expected_m)
    }

    #[test]
    fn test_deserialize_metadata_skip_lib() {
        // given:
        let string = r#"{
            "name": "pkg1",
            "version": "0.0.1",
            "depends": "libsome"
        }"#;
        let expected_m = Metadata {
            name: String::from("pkg1"),
            version: String::from("0.0.1"),
            requires_dist: vec![],
        };
        // when:
        let m: Metadata = serde_json::from_str(string).unwrap();
        // then:
        assert_eq!(m, expected_m)
    }

    #[test]
    fn test_deserialize_metadata_skip_underscore() {
        // given:
        let string = r#"{
            "name": "pkg1",
            "version": "0.0.1",
            "depends": "_liblowlevel"
        }"#;
        let expected_m = Metadata {
            name: String::from("pkg1"),
            version: String::from("0.0.1"),
            requires_dist: vec![],
        };
        // when:
        let m: Metadata = serde_json::from_str(string).unwrap();
        // then:
        assert_eq!(m, expected_m)
    }

    #[test]
    fn test_deserialize_metadata_depends_list() {
        // given:
        let string = r#"{
            "name": "pkg1",
            "version": "0.0.1",
            "depends": ["pkg2a", "pkg2b"]
        }"#;
        let expected_m = Metadata {
            name: String::from("pkg1"),
            version: String::from("0.0.1"),
            requires_dist: vec![String::from("pkg2a"), String::from("pkg2b")],
        };
        // when:
        let m: Metadata = serde_json::from_str(string).unwrap();
        // then:
        assert_eq!(m, expected_m)
    }

    #[test]
    fn test_deserialize_metadata_depends_list_skip_python_and_low_level() {
        // given:
        let string = r#"{
            "name": "pkg1",
            "version": "0.0.1",
            "depends": ["pkg2a", "pkg2b", "python", "libsome", "_liblowlevel"]
        }"#;
        let expected_m = Metadata {
            name: String::from("pkg1"),
            version: String::from("0.0.1"),
            requires_dist: vec![String::from("pkg2a"), String::from("pkg2b")],
        };
        // when:
        let m: Metadata = serde_json::from_str(string).unwrap();
        // then:
        assert_eq!(m, expected_m)
    }
}
