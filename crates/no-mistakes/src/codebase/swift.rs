mod manifest;
mod scanner;
mod source;

use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

pub(crate) use manifest::extract_test_target_names;
use manifest::parse_manifest_targets;
use source::parse_swift_file;

#[derive(Debug, Clone, Default)]
pub(crate) struct SwiftPackageFacts {
    pub package_root: PathBuf,
    pub targets: BTreeMap<String, SwiftTargetFacts>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SwiftTargetFacts {
    pub name: String,
    pub is_test: bool,
    pub dependencies: Vec<String>,
    pub roots: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SwiftFileFacts {
    pub path: PathBuf,
    pub target: Option<String>,
    pub imports: Vec<String>,
    pub declarations: Vec<String>,
    pub references: Vec<String>,
    pub endpoint_paths: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SwiftFactMap {
    pub packages: Vec<SwiftPackageFacts>,
    pub files: BTreeMap<PathBuf, SwiftFileFacts>,
    pub declarations: HashMap<String, BTreeSet<PathBuf>>,
    pub files_by_target: HashMap<String, BTreeSet<PathBuf>>,
}

pub(crate) fn collect_swift_facts(
    root: &Path,
    all_files: &[PathBuf],
    packages: &[String],
) -> SwiftFactMap {
    if packages.is_empty() {
        return SwiftFactMap::default();
    }
    let package_facts: Vec<SwiftPackageFacts> = packages
        .iter()
        .filter_map(|package| parse_package(root, package))
        .collect();
    if package_facts.is_empty() {
        return SwiftFactMap::default();
    }
    let swift_files: Vec<PathBuf> = all_files
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("swift"))
        .filter(|path| {
            package_facts
                .iter()
                .any(|package| path.starts_with(&package.package_root))
        })
        .cloned()
        .collect();

    let target_by_file = target_index(&package_facts, &swift_files);
    let mut file_facts: Vec<SwiftFileFacts> = swift_files
        .par_iter()
        .filter_map(|path| parse_swift_file(path, target_by_file.get(path).cloned()))
        .collect();
    file_facts.sort_by(|a, b| a.path.cmp(&b.path));

    let mut facts = SwiftFactMap {
        packages: package_facts,
        ..SwiftFactMap::default()
    };
    for file in file_facts {
        if let Some(target) = &file.target {
            facts
                .files_by_target
                .entry(target.clone())
                .or_default()
                .insert(file.path.clone());
        }
        for declaration in &file.declarations {
            facts
                .declarations
                .entry(declaration.clone())
                .or_default()
                .insert(file.path.clone());
        }
        facts.files.insert(file.path.clone(), file);
    }
    facts
}

fn parse_package(root: &Path, package: &str) -> Option<SwiftPackageFacts> {
    let package_rel = package.trim_end_matches('/').to_string();
    let package_root = root.join(&package_rel);
    let manifest = package_root.join("Package.swift");
    let source = std::fs::read_to_string(manifest).ok()?;
    let mut targets = BTreeMap::new();
    for target in parse_manifest_targets(&source) {
        targets.insert(target.name.clone(), target);
    }
    for target in targets.values_mut() {
        let default_root = if target.is_test {
            package_root.join("Tests").join(&target.name)
        } else {
            package_root.join("Sources").join(&target.name)
        };
        target.roots.push(default_root);
    }
    Some(SwiftPackageFacts {
        package_root,
        targets,
    })
}

fn target_index(
    packages: &[SwiftPackageFacts],
    swift_files: &[PathBuf],
) -> HashMap<PathBuf, String> {
    let mut index = HashMap::new();
    for file in swift_files {
        let mut best: Option<(&String, usize)> = None;
        for package in packages {
            for (name, target) in &package.targets {
                for root in &target.roots {
                    if file.starts_with(root) {
                        let depth = root.components().count();
                        if best.is_none_or(|(_, best_depth)| depth > best_depth) {
                            best = Some((name, depth));
                        }
                    }
                }
            }
        }
        if let Some((name, _)) = best {
            index.insert(file.clone(), name.clone());
        }
    }
    index
}
