use regex::Regex;
use std::sync::OnceLock;

mod manifest;

pub use manifest::{
    parse_cargo_bins, parse_cargo_package_name, parse_cargo_workspace_excludes,
    parse_cargo_workspace_members,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoTarget {
    pub package: Option<String>,
    pub binary: String,
}

pub fn extract_binary_names(run: &str) -> Vec<String> {
    static TARGET_BIN: OnceLock<Regex> = OnceLock::new();

    let target_bin_re = TARGET_BIN.get_or_init(|| {
        Regex::new(r"(?:\./)?target/(?:[\w.-]+/)*(?:debug|release)/([\w-]+)")
            .expect("Failed to compile target binary regex pattern")
    });

    let mut names: Vec<String> = extract_cargo_targets(run)
        .into_iter()
        .map(|target| target.binary)
        .collect();
    for cap in target_bin_re.captures_iter(run) {
        names.push(cap[1].to_string());
    }
    names.sort();
    names.dedup();
    names
}

pub fn extract_cargo_targets(run: &str) -> Vec<CargoTarget> {
    let tokens = shellish_words(run);
    let mut targets = Vec::new();
    let mut i = 0;
    while i + 1 < tokens.len() {
        if tokens[i] != "cargo" || !is_cargo_binary_subcommand(&tokens[i + 1]) {
            i += 1;
            continue;
        }

        i += 2;
        let mut package: Option<String> = None;
        let mut bins: Vec<String> = Vec::new();
        while i < tokens.len() {
            match tokens[i].as_str() {
                "--" => {
                    i += 1;
                    break;
                }
                "cargo" if i + 1 < tokens.len() && is_cargo_binary_subcommand(&tokens[i + 1]) => {
                    break;
                }
                "--bin" => {
                    if let Some(name) = tokens.get(i + 1).filter(|name| is_cargo_target_name(name))
                    {
                        bins.push(name.clone());
                    }
                    i += 2;
                }
                "-p" | "--package" => {
                    if let Some(name) = tokens.get(i + 1).filter(|name| is_cargo_target_name(name))
                    {
                        package = Some(name.clone());
                    }
                    i += 2;
                }
                token if token.starts_with("--bin=") => {
                    let name = token.trim_start_matches("--bin=");
                    if is_cargo_target_name(name) {
                        bins.push(name.to_string());
                    }
                    i += 1;
                }
                token if token.starts_with("-p=") => {
                    let name = token.trim_start_matches("-p=");
                    if is_cargo_target_name(name) {
                        package = Some(name.to_string());
                    }
                    i += 1;
                }
                token if token.starts_with("--package=") => {
                    let name = token.trim_start_matches("--package=");
                    if is_cargo_target_name(name) {
                        package = Some(name.to_string());
                    }
                    i += 1;
                }
                _ => i += 1,
            }
        }

        if bins.is_empty() {
            if let Some(package) = package {
                targets.push(CargoTarget {
                    package: Some(package.clone()),
                    binary: package,
                });
            }
        } else {
            targets.extend(bins.into_iter().map(|binary| CargoTarget {
                package: package.clone(),
                binary,
            }));
        }
    }
    targets
}

fn shellish_words(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .filter_map(|token| {
            let token = token.trim_matches(|c| matches!(c, '"' | '\'' | ';' | '\\'));
            (!token.is_empty()).then(|| token.to_string())
        })
        .collect()
}

fn is_cargo_binary_subcommand(token: &str) -> bool {
    matches!(token, "run" | "build" | "test")
}

fn is_cargo_target_name(token: &str) -> bool {
    token
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}
