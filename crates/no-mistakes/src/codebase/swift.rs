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
    let mut names: Vec<String> = parse_manifest_targets(package_swift)
        .into_iter()
        .filter(|target| target.is_test)
        .map(|target| target.name)
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
    let Ok(re) = Regex::new(r#"\.(target|testTarget)\s*\("#) else {
        return Vec::new();
    };
    let name_re = Regex::new(r#"name\s*:\s*\"([^\"]+)\""#).expect("valid name regex");
    re.captures_iter(source)
        .filter_map(|cap| {
            let kind = cap.get(1)?.as_str();
            let call = cap.get(0)?;
            let body_end = find_matching_delimiter(source, call.end().checked_sub(1)?, '(', ')')?;
            let body = &source[call.start()..=body_end];
            let name = name_re.captures(body)?.get(1)?.as_str().to_string();
            let dependencies = manifest_dependencies_body(body)
                .map(manifest_dependency_names)
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

fn manifest_dependencies_body(target_body: &str) -> Option<&str> {
    let deps_re = Regex::new(r#"dependencies\s*:\s*\["#).expect("valid dependencies regex");
    let deps = deps_re.find(target_body)?;
    let open_bracket = deps.end().checked_sub(1)?;
    let close_bracket = find_matching_delimiter(target_body, open_bracket, '[', ']')?;
    target_body.get(open_bracket + 1..close_bracket)
}

fn manifest_dependency_names(dependencies_body: &str) -> Vec<String> {
    let name_re = Regex::new(r#"name\s*:\s*\"([^\"]+)\""#).expect("valid dependency name regex");
    let mut names = Vec::new();
    let mut index = 0usize;

    while index < dependencies_body.len() {
        let rest = &dependencies_body[index..];
        if rest.starts_with(".target") || rest.starts_with(".product") {
            let Some(open_rel) = rest.find('(') else {
                break;
            };
            let open = index + open_rel;
            let Some(close) = find_matching_delimiter(dependencies_body, open, '(', ')') else {
                break;
            };
            if let Some(name) = name_re
                .captures(&dependencies_body[index..=close])
                .and_then(|cap| cap.get(1))
            {
                names.push(name.as_str().to_string());
            }
            index = close + 1;
            continue;
        }

        let Some(ch) = rest.chars().next() else {
            break;
        };
        if ch == '"' {
            if let Some((value, next)) = read_quoted_string(dependencies_body, index) {
                names.push(value);
                index = next;
                continue;
            }
        }
        index += ch.len_utf8();
    }

    names
}

fn read_quoted_string(source: &str, quote_index: usize) -> Option<(String, usize)> {
    let mut value = String::new();
    let mut escaped = false;
    for (offset, ch) in source[quote_index + 1..].char_indices() {
        if escaped {
            value.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
            value.push(ch);
        } else if ch == '"' {
            return Some((value, quote_index + 1 + offset + ch.len_utf8()));
        } else {
            value.push(ch);
        }
    }
    None
}

fn find_matching_delimiter(
    source: &str,
    open_index: usize,
    open_char: char,
    close_char: char,
) -> Option<usize> {
    let mut chars = source.char_indices().peekable();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while let Some((index, ch)) = chars.next() {
        if index < open_index {
            continue;
        }

        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            continue;
        }
        if in_block_comment {
            if ch == '*' && chars.peek().is_some_and(|(_, next)| *next == '/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '/') {
            chars.next();
            in_line_comment = true;
            continue;
        }
        if ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '*') {
            chars.next();
            in_block_comment = true;
            continue;
        }
        if ch == '"' {
            in_string = true;
            continue;
        }
        if ch == open_char {
            depth += 1;
        } else if ch == close_char {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
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
    let re = Regex::new(r#"(?s)(\"(?:\\.|[^\"\\])*\")|/\*.*?\*/|//[^\n]*"#)
        .expect("valid comment regex");
    re.replace_all(source, |caps: &regex::Captures<'_>| {
        caps.get(1)
            .map(|mat| mat.as_str().to_string())
            .unwrap_or_else(|| " ".to_string())
    })
    .into_owned()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_targets_handle_nested_dependency_parentheses() {
        let source = r#"
            let package = Package(
                name: "Fixture",
                targets: [
                    .target(
                        name: "VouchaFeatures",
                        dependencies: [
                            .product(name: "VouchaCore", package: "core"),
                            "VouchaAPI",
                        ]
                    ),
                    .testTarget(
                        name: "VouchaUITests",
                        dependencies: [
                            .target(name: "VouchaFeatures"),
                            .product(name: "VouchaModels", package: "core"),
                        ]
                    ),
                ]
            )
        "#;

        let targets = parse_manifest_targets(source);
        let features = targets
            .iter()
            .find(|target| target.name == "VouchaFeatures")
            .expect("source target should parse");
        assert_eq!(
            features.dependencies,
            vec!["VouchaCore".to_string(), "VouchaAPI".to_string()]
        );

        let ui_tests = targets
            .iter()
            .find(|target| target.name == "VouchaUITests")
            .expect("test target should parse");
        assert!(ui_tests.is_test);
        assert_eq!(
            ui_tests.dependencies,
            vec!["VouchaFeatures".to_string(), "VouchaModels".to_string()]
        );
        assert_eq!(
            extract_test_target_names(source),
            vec!["VouchaUITests".to_string()]
        );
    }

    #[test]
    fn comment_stripping_preserves_comment_markers_inside_strings() {
        let source = r#"
            let site = "https://example.com/feed"
            static let rss = Endpoint(path: "/api/v1/feeds/rss_feed_items/\(feedType)")
            // Endpoint(path: "/api/v1/commented")
            let marker = "not /* a comment */"
            /* Endpoint(path: "/api/v1/blocked") */
        "#;

        let stripped = strip_comments(source);
        assert!(stripped.contains(r#""https://example.com/feed""#));
        assert!(stripped.contains(r#""/api/v1/feeds/rss_feed_items/\(feedType)""#));
        assert!(stripped.contains(r#""not /* a comment */""#));
        assert_eq!(
            extract_endpoint_paths(&stripped),
            vec!["/api/v1/feeds/rss_feed_items/*".to_string()]
        );
    }
}
