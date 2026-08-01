//! Orchestrates manual `package.json` discovery, import extraction,
//! production-reachability closure, and finding emission for one rule
//! application.

use super::discovery;
use super::manifest::{Classification, PackageManifest};
use super::reachability::{self, FileImport};
use super::specifier;
use super::Options;
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

const DEFAULT_ALLOWED_FIELDS: &[&str] =
    &["dependencies", "optionalDependencies", "peerDependencies"];
const DEFAULT_TEST_FILE_PATTERNS: &[&str] = &["**/__tests__/**", "**/*.test.*", "**/*.d.*ts"];

pub(super) fn run(
    root: &Path,
    workspace_roots: &[PathBuf],
    opts: &Options,
    files: &[PathBuf],
    sources: &SourceStore,
) -> anyhow::Result<Vec<RuleFinding>> {
    let allowed_fields = match allowed_fields(opts) {
        Ok(fields) => fields,
        Err(message) => return Ok(vec![super::findings::config(&message)]),
    };
    let test_globset = match build_globset(&test_file_patterns(opts)) {
        Ok(globset) => globset,
        Err(error) => {
            return Ok(vec![super::findings::config(&format!(
                "invalid glob pattern in testFilePatterns: {error}"
            ))]);
        }
    };

    let workspace = discovery::load_workspace(root, workspace_roots, files, sources)?;
    if workspace.packages.is_empty() {
        return Ok(Vec::new());
    }

    let visible: HashSet<PathBuf> = files.iter().map(|path| normalize_path(path)).collect();
    let imports_by_file: HashMap<PathBuf, Vec<FileImport>> = files
        .iter()
        .map(|file| {
            (
                normalize_path(file),
                reachability::file_imports(file, sources),
            )
        })
        .collect();
    let owners = discovery::compute_owners(&workspace, files);
    let package_files = discovery::group_by_package(&owners);
    let reachability_ctx = reachability::ReachabilityContext {
        root,
        workspace: &workspace,
        imports_by_file: &imports_by_file,
        owners: &owners,
        test_globset: &test_globset,
        visible: &visible,
    };

    let mut findings = Vec::new();
    for package in &workspace.packages {
        let package_dir = normalize_path(&package.dir);
        let Some(files_in_package) = package_files.get(&package_dir) else {
            continue;
        };
        let manifest = PackageManifest::load(&package.dir.join("package.json"), sources);
        let reachable = reachability::production_reachable_files(
            &reachability_ctx,
            &package_dir,
            files_in_package,
        );
        for file in &reachable {
            if test_globset.is_match(relative_slash_path(root, file)) {
                continue;
            }
            let Some(imports) = imports_by_file.get(file) else {
                continue;
            };
            for import in imports {
                emit_finding(
                    root,
                    file,
                    import,
                    &package.name,
                    &manifest,
                    &allowed_fields,
                    &mut findings,
                );
            }
        }
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

fn emit_finding(
    root: &Path,
    file: &Path,
    import: &FileImport,
    owning_package: &str,
    manifest: &PackageManifest,
    allowed_fields: &BTreeSet<String>,
    findings: &mut Vec<RuleFinding>,
) {
    if import.kind == ImportKind::Type {
        return; // erased by verbatimModuleSyntax; cannot fail at runtime
    }
    if specifier::is_relative(&import.specifier) || import.specifier.starts_with('#') {
        return; // not a package.json dependency question
    }
    let Some(package_name) = specifier::package_name(&import.specifier) else {
        return;
    };
    if package_name == owning_package || specifier::is_node_builtin(&package_name) {
        return; // self-reference and Node builtins never need a declaration
    }
    match manifest.classify(&package_name, allowed_fields) {
        Classification::Allowed => {}
        Classification::DevOnly => findings.push(super::findings::dev_only(
            root,
            file,
            import.line,
            &package_name,
            &import.specifier,
        )),
        Classification::Undeclared => findings.push(super::findings::undeclared(
            root,
            file,
            import.line,
            &package_name,
            &import.specifier,
        )),
    }
}

fn allowed_fields(opts: &Options) -> Result<BTreeSet<String>, String> {
    if opts.allowed_fields.is_empty() {
        return Ok(DEFAULT_ALLOWED_FIELDS
            .iter()
            .map(|s| s.to_string())
            .collect());
    }
    let mut validated = BTreeSet::new();
    for field in &opts.allowed_fields {
        if !crate::codebase::package_deps::ALL_DEPENDENCY_FIELDS.contains(&field.as_str()) {
            return Err(format!(
                "allowedFields supports dependencies, devDependencies, peerDependencies, and \
                 optionalDependencies only; unsupported field '{field}'"
            ));
        }
        validated.insert(field.clone());
    }
    Ok(validated)
}

fn test_file_patterns(opts: &Options) -> Vec<String> {
    if opts.test_file_patterns.is_empty() {
        DEFAULT_TEST_FILE_PATTERNS
            .iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        opts.test_file_patterns.clone()
    }
}

fn build_globset(patterns: &[String]) -> Result<GlobSet, globset::Error> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    builder.build()
}

#[cfg(test)]
mod tests;
