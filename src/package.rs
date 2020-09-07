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
            .map(|name| Metadata::from_name(name).unwrap())
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

impl Into<String> for Package {
    fn into(self) -> String {
        match &self.installer {
            Installer::Pip => return format!("{}=={}", self.name, self.version),
            Installer::Conda => return format!("{}={}", self.name, self.version),
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
            String::from("package"),
            String::from("1.0.0"),
            vec![],
            Installer::default(),
        );
        let package_str = format!("{}", package);
        assert_eq!(package_str, "package (v1.0.0)".to_string())
    }

    #[test]
    fn test_package_display_any_version() {
        let package = Package::new(
            String::from("package"),
            String::from("any"),
            vec![],
            Installer::default(),
        );
        let package_str = format!("{}", package);
        assert_eq!(package_str, "package".to_string())
    }

    #[test]
    fn test_convert_from_pipmetadata_to_package() {
        let metadata = Metadata::new(String::from("some_package"), String::from("1.0.0"), vec![]);
        let expected_package = Package {
            name: String::from("some_package"),
            version: String::from("1.0.0"),
            requires: vec![],
            installer: Installer::default(),
        };
        assert_eq!(Package::from(metadata), expected_package)
    }

    #[test]
    fn test_into_string_conda() {
        let p: String = Package::new(
            String::from("conda1"),
            String::from("0.0.1"),
            vec![],
            Installer::Conda,
        )
        .into();
        assert_eq!(p, String::from("conda1=0.0.1"))
    }

    #[test]
    fn test_into_string_pip() {
        let p: String = Package::new(
            String::from("pip1"),
            String::from("0.0.1"),
            vec![],
            Installer::Pip,
        )
        .into();
        assert_eq!(p, String::from("pip1==0.0.1"))
    }
}
