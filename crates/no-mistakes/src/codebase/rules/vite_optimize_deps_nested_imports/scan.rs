//! Compare nested include entries with declared dependencies and entry imports.

use super::include_config::{parse_vite_config, NestedInclude, ParsedConfig};
use super::include_source;
use super::manifest::{self, PackageFacts};
use super::specifier::{is_node_builtin, package_name};
use super::{IncludeSource, RuleFinding, RULE_ID};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) fn evaluate(
    root: &Path,
    sources: &SourceStore,
    config_files: &[String],
    package_dirs: &[String],
    all_files: &[PathBuf],
    include_sources: &[IncludeSource],
    ignore_packages: &[String],
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    let mut includes = Vec::new();
    let mut configs_known = true;
    let allow_imported = !include_sources.is_empty();
    for config_file in config_files {
        let path = resolve(root, config_file);
        let file = relative_slash_path(root, &path);
        match sources.read_source_path(&path) {
            Ok(source) => match parse_vite_config(&path, &source, allow_imported) {
                ParsedConfig::Includes(entries) => {
                    includes.extend(entries.into_iter().map(|entry| (file.clone(), entry)));
                }
                ParsedConfig::Unanalyzable { line, reason } => {
                    configs_known = false;
                    findings.push(config_gap(&file, line, &reason));
                }
            },
            Err(error) => {
                configs_known = false;
                findings.push(super::super::source_access::read_failure(
                    root, &path, RULE_ID, &error,
                ));
            }
        }
    }
    include_source::load(
        root,
        sources,
        include_sources,
        &mut includes,
        &mut findings,
        &mut configs_known,
    );
    let mut packages = BTreeMap::new();
    for package_dir in package_dirs {
        match manifest::load(root, &resolve(root, package_dir), sources, all_files) {
            Ok(package) => {
                if packages.contains_key(&package.name) {
                    let manifest = resolve(root, package_dir).join("package.json");
                    findings.push(duplicate_package(root, &manifest, &package.name));
                    continue;
                }
                packages.insert(package.name.clone(), package);
            }
            Err(finding) => findings.push(*finding),
        }
    }
    findings.extend(invalid_mappings(&includes, &packages));
    if configs_known {
        let ignore: BTreeSet<_> = ignore_packages.iter().cloned().collect();
        let configured: BTreeSet<_> = packages.keys().cloned().collect();
        findings.extend(uncovered(&includes, &packages, &configured, &ignore));
    }
    Ok(findings)
}

fn invalid_mappings(
    includes: &[(String, NestedInclude)],
    packages: &BTreeMap<String, PackageFacts>,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for (file, entry) in includes {
        let Some(package) = packages.get(&entry.parent) else {
            continue;
        };
        let Some(declared_name) = package_name(&entry.child) else {
            continue;
        };
        if package.declared.contains(&declared_name) || is_node_builtin(&entry.child) {
            continue;
        }
        findings.push(RuleFinding {
            causes: Vec::new(),
            rule: RULE_ID.to_string(),
            file: file.clone(),
            line: entry.line,
            message: format!(
                "nested include \"{}\" is not a declared dependency of {}; declare {} under dependencies, optionalDependencies, peerDependencies, or bundledDependencies, or remove the include entry",
                entry.text, entry.parent, declared_name
            ),
            import: Some(entry.text.clone()),
            target: Some("invalid-nested-child".to_string()),
        });
    }
    findings
}

fn uncovered(
    includes: &[(String, NestedInclude)],
    packages: &BTreeMap<String, PackageFacts>,
    configured: &BTreeSet<String>,
    ignore: &BTreeSet<String>,
) -> Vec<RuleFinding> {
    let covered: BTreeSet<_> = includes
        .iter()
        .map(|(_, entry)| (entry.parent.clone(), entry.child.clone()))
        .collect();
    let mut findings = Vec::new();
    for package in packages.values() {
        if let Some(unknown) = &package.unknown {
            findings.push(unknown.clone());
            continue;
        }
        for import in &package.imports {
            if covered.contains(&(package.name.clone(), import.specifier.clone()))
                || skip_child(package, &import.package, configured, ignore)
            {
                continue;
            }
            findings.push(RuleFinding {
                causes: Vec::new(),
                rule: RULE_ID.to_string(),
                file: import.file.clone(),
                line: import.line,
                message: format!(
                    "runtime import \"{}\" from {} is not covered by optimizeDeps.include; add \"{} > {}\"",
                    import.specifier, package.name, package.name, import.specifier
                ),
                import: Some(format!("{} > {}", package.name, import.specifier)),
                target: Some("uncovered-runtime-import".to_string()),
            });
        }
    }
    findings
}

fn skip_child(
    package: &PackageFacts,
    child: &str,
    configured: &BTreeSet<String>,
    ignore: &BTreeSet<String>,
) -> bool {
    ignore.contains(child)
        || package.workspace.contains(child)
        || (configured.contains(child) && !package.versioned.contains(child))
}

fn duplicate_package(root: &Path, path: &Path, name: &str) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, path),
        line: 1,
        message: format!(
            "cannot verify {RULE_ID}: package name \"{name}\" is configured more than once; keep one packages entry per name so this check can verify nested imports"
        ),
        import: None,
        target: Some("unanalyzable".to_string()),
    }
}

fn config_gap(file: &str, line: usize, reason: &str) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!(
            "cannot verify {RULE_ID}: {reason}; export a static optimizeDeps.include array of string literals so this check can verify nested imports"
        ),
        import: None,
        target: Some("unanalyzable".to_string()),
    }
}

fn resolve(root: &Path, value: &str) -> PathBuf {
    // `Path::join` keeps an absolute configured path instead of prefixing root.
    crate::codebase::ts_resolver::normalize_path(&root.join(value))
}
