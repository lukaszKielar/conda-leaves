use std::convert::From;
use std::fmt;

use crate::metadata::Metadata;

#[derive(Debug, Clone, PartialEq)]
pub enum Installer {
    Pip,
    Conda,
}

impl Default for Installer {
    fn default() -> Self {
        Installer::Conda
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    name: String,
    version: String,
    requires: Vec<Package>,
    installer: Installer,
}

impl Package {
    pub fn new(
        name: String,
        version: String,
        requires: Vec<Package>,
        installer: Installer,
    ) -> Self {
        Package {
            name,
            version,
            requires,
            installer,
        }
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let package_string = if self.version == "any" {
            format!("{}", self.name)
        } else {
            format!("{} (v{})", self.name, self.version)
        };
        write!(f, "{}", package_string)?;
        Ok(())
    }
}

impl From<Metadata> for Package {
    fn from(metadata: Metadata) -> Self {
        // get list of packages from Metadata.requires_dist
        let requires: Vec<Package> = metadata
            .requires_dist
            .iter()
            .map(|name| Metadata::from_name(name.to_string()).unwrap())
            .map(|m| Package::from(m))
            .collect();
        // TODO add required by
        Package {
            name: metadata.name,
            version: metadata.version,
            requires: requires,
            installer: Installer::default(),
        }
    }
}

pub fn print_package(package: &Package) {
    let tree = package_to_lines(package).join("\n");
    println!("{}", tree)
}

pub fn package_to_lines(package: &Package) -> Vec<String> {
    let mut lines = vec![format!("{}", package)];
    let children = &package.requires[..];
    if let Some((last_child, non_last_children)) = children.split_last() {
        let child_node_lines = non_last_children.iter().flat_map(|child| {
            package_to_lines(child)
                .iter()
                .enumerate()
                .map(|(idx, child_line)| {
                    if idx == 0 {
                        format!("├── {}", child_line)
                    } else {
                        format!("│   {}", child_line)
                    }
                })
                .collect::<Vec<String>>()
        });
        let last_child_node_lines = package_to_lines(last_child);
        let formatted_last_child_node_lines_iter =
            last_child_node_lines
                .iter()
                .enumerate()
                .map(|(idx, child_line)| {
                    if idx == 0 {
                        format!("└── {}", child_line)
                    } else {
                        format!("    {}", child_line)
                    }
                });
        let children_lines = child_node_lines.chain(formatted_last_child_node_lines_iter);
        lines.extend(children_lines);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_display_with_version() {
        let package = Package::new(
            "package".to_string(),
            "1.0.0".to_string(),
            vec![],
            Installer::default(),
        );
        let package_str = format!("{}", package);
        assert_eq!(package_str, "package (v1.0.0)".to_string())
    }

    #[test]
    fn test_package_display_any_version() {
        let package = Package::new(
            "package".to_string(),
            "any".to_string(),
            vec![],
            Installer::default(),
        );
        let package_str = format!("{}", package);
        assert_eq!(package_str, "package".to_string())
    }

    #[test]
    fn test_convert_from_pipmetadata_to_package() {
        let metadata = Metadata::new("some_package".to_string(), "1.0.0".to_string(), vec![]);
        let expected_package = Package {
            name: "some_package".to_string(),
            version: "1.0.0".to_string(),
            requires: vec![],
            installer: Installer::default(),
        };
        assert_eq!(Package::from(metadata), expected_package)
    }
}
