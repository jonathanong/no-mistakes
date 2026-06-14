use rayon::prelude::*;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

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

pub(crate) fn extract_test_target_names(package_swift: &str) -> Vec<String> {
    let Ok(re) = Regex::new(r#"\.testTarget\s*\([^)]*name\s*:\s*\"([^\"]+)\""#) else {
        return Vec::new();
    };
    let mut names: Vec<String> = re
        .captures_iter(package_swift)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect();
    names.sort();
    names.dedup();
    names
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

fn parse_manifest_targets(source: &str) -> Vec<SwiftTargetFacts> {
    let Ok(re) = Regex::new(r#"\.(target|testTarget)\s*\((?s:.*?)\)"#) else {
        return Vec::new();
    };
    let name_re = Regex::new(r#"name\s*:\s*\"([^\"]+)\""#).expect("valid name regex");
    let deps_re = Regex::new(r#"dependencies\s*:\s*\[(?s:(.*?))\]"#).expect("valid deps regex");
    let dep_name_re =
        Regex::new(r#"(?:\.target\s*\(|\.product\s*\()?\s*name\s*:\s*\"([^\"]+)\"|\"([^\"]+)\""#)
            .expect("valid dependency regex");
    re.captures_iter(source)
        .filter_map(|cap| {
            let kind = cap.get(1)?.as_str();
            let body = cap.get(0)?.as_str();
            let name = name_re.captures(body)?.get(1)?.as_str().to_string();
            let dependencies = deps_re
                .captures(body)
                .and_then(|deps| deps.get(1))
                .map(|deps| {
                    dep_name_re
                        .captures_iter(deps.as_str())
                        .filter_map(|dep| {
                            dep.get(1)
                                .or_else(|| dep.get(2))
                                .map(|m| m.as_str().to_string())
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Some(SwiftTargetFacts {
                name,
                is_test: kind == "testTarget",
                dependencies,
                roots: Vec::new(),
            })
        })
        .collect()
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

fn parse_swift_file(path: &Path, target: Option<String>) -> Option<SwiftFileFacts> {
    let source = std::fs::read_to_string(path).ok()?;
    let stripped = strip_comments(&source);
    Some(SwiftFileFacts {
        path: path.to_path_buf(),
        target,
        imports: extract_imports(&stripped),
        declarations: extract_declarations(&stripped),
        references: extract_references(&stripped),
        endpoint_paths: extract_endpoint_paths(&stripped),
    })
}

fn strip_comments(source: &str) -> String {
    let line_re = Regex::new(r"//.*").expect("valid line comment regex");
    let block_re = Regex::new(r"(?s)/\*.*?\*/").expect("valid block comment regex");
    let source = block_re.replace_all(source, " ");
    line_re.replace_all(&source, " ").into_owned()
}

fn extract_imports(source: &str) -> Vec<String> {
    let re = Regex::new(r"(?m)^\s*import\s+([A-Za-z_][A-Za-z0-9_]*)").expect("valid import regex");
    sorted_unique(
        re.captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    )
}

fn extract_declarations(source: &str) -> Vec<String> {
    let decl_re = Regex::new(r"\b(?:public\s+|internal\s+|private\s+|fileprivate\s+|open\s+|final\s+|static\s+|class\s+)*\b(?:struct|class|actor|enum|protocol|extension|typealias)\s+([A-Za-z_][A-Za-z0-9_]*)")
        .expect("valid declaration regex");
    let func_re = Regex::new(r"\b(?:static\s+|class\s+)?func\s+([A-Za-z_][A-Za-z0-9_]*)")
        .expect("valid function regex");
    let let_re = Regex::new(r"\b(?:static\s+|class\s+)?(?:let|var)\s+([A-Za-z_][A-Za-z0-9_]*)")
        .expect("valid property regex");
    let mut out = Vec::new();
    out.extend(
        decl_re
            .captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    );
    out.extend(
        func_re
            .captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    );
    out.extend(
        let_re
            .captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    );
    sorted_unique(out)
}

fn extract_references(source: &str) -> Vec<String> {
    let ident_re = Regex::new(r"\b[A-Z_][A-Za-z0-9_]*\b|\.[A-Za-z_][A-Za-z0-9_]*\b")
        .expect("valid reference regex");
    let keywords: HashSet<&str> = [
        "Array",
        "Bool",
        "Data",
        "Dictionary",
        "Double",
        "Error",
        "False",
        "Float",
        "Int",
        "Nil",
        "Optional",
        "Result",
        "Self",
        "Set",
        "String",
        "True",
        "Void",
    ]
    .into_iter()
    .collect();
    sorted_unique(ident_re.captures_iter(source).filter_map(|cap| {
        let raw = cap.get(0)?.as_str().trim_start_matches('.');
        (!keywords.contains(raw)).then(|| raw.to_string())
    }))
}

fn extract_endpoint_paths(source: &str) -> Vec<String> {
    let re = Regex::new(r#"path\s*:\s*\"([^\"]+)\""#).expect("valid endpoint path regex");
    sorted_unique(
        re.captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| swift_path_pattern(m.as_str()))),
    )
}

fn swift_path_pattern(path: &str) -> String {
    let interpolation = Regex::new(r#"\\\([^)]*\)"#).expect("valid interpolation regex");
    interpolation.replace_all(path, "*").into_owned()
}

fn sorted_unique<I>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut out: Vec<String> = values.into_iter().collect();
    out.sort();
    out.dedup();
    out
}
