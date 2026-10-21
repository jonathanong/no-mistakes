//! Check-rule snippet execution, extracted from docs_coverage.rs.

#[path = "support/docs_rule_snippets.rs"]
mod docs_rule_snippets;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

fn rust_sources(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_rust_files(dir, &mut paths);
    paths.sort();
    paths
}

fn collect_rust_files(dir: &Path, paths: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, paths);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            paths.push(path);
        }
    }
}

fn rule_ids_from_source(root: &Path) -> BTreeSet<String> {
    let rules_dir = root.join("crates/no-mistakes/src/codebase/rules");
    let mut sources = rust_sources(&rules_dir);
    sources.extend([
        root.join("crates/no-mistakes/src/codebase/unique_exports.rs"),
        root.join("crates/no-mistakes/src/playwright/rules.rs"),
    ]);

    sources
        .into_iter()
        .flat_map(|path| {
            read(&path)
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    let is_rule_constant = line.starts_with("pub const RULE_ID: &str = \"")
                        || line.starts_with("pub const REQUIRED_DOC_SECTION_RULE_ID: &str = \"")
                        || line.starts_with("pub const PLAYWRIGHT_");
                    is_rule_constant
                        .then(|| line.split_once("= \"")?.1.split_once('"').map(|(id, _)| id))
                        .flatten()
                        .map(str::to_string)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn rule_docs_execute_valid_and_counterexample_snippets() {
    let root = repo_root();
    let mut failures = Vec::new();
    for rule_id in rule_ids_from_source(&root) {
        if docs_rule_snippets::skip_open_pr(&rule_id) {
            continue;
        }
        let page = docs_rule_snippets::load_page(&root, &rule_id);
        let mut valid = docs_rule_snippets::heading_section(&page.body, "Valid example");
        if docs_rule_snippets::fences(&valid).is_empty() {
            valid = docs_rule_snippets::labeled_section(&page.body, &["Valid example", "Pass"]);
        }
        let mut counter = docs_rule_snippets::counterexample_section(&page.body);
        if docs_rule_snippets::fences(&counter).is_empty() {
            counter = docs_rule_snippets::labeled_section(&page.body, &["Counterexample", "Fail"]);
        }
        let valid_fences = docs_rule_snippets::fences(&valid);
        let counter_fences = docs_rule_snippets::fences(&counter);
        if valid_fences.is_empty() {
            failures.push(format!(
                "{}: Valid example needs an executable fence",
                page.file
            ));
            continue;
        }
        if counter_fences.is_empty() {
            failures.push(format!(
                "{}: Counterexample needs an executable fence",
                page.file
            ));
            continue;
        }
        let config = docs_rule_snippets::rule_config_yaml(&page.body, &rule_id);
        for (label, expect_report, snippet) in [
            ("Valid example", false, valid_fences.as_slice()),
            ("Counterexample", true, counter_fences.as_slice()),
        ] {
            for (index, fence) in snippet.iter().enumerate() {
                let fixture =
                    docs_rule_snippets::materialize_snippet(&config, std::slice::from_ref(fence));
                match docs_rule_snippets::run_rule(fixture.path(), &rule_id) {
                    Ok(hits) => {
                        if hits.is_empty() == expect_report {
                            failures.push(format!(
                                "{}: {label}[{index}] expected {} but hits={hits:?}",
                                page.file,
                                if expect_report {
                                    "a finding"
                                } else {
                                    "silence"
                                },
                            ));
                        }
                    }
                    Err(err) => {
                        failures.push(format!("{}: {label}[{index}] run error: {err}", page.file))
                    }
                }
            }
        }
    }
    let gaps = read(&root.join("crates/no-mistakes/tests/support/docs_rule_snippet_gaps.txt"));
    let known: Vec<&str> = gaps
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    let unexpected: Vec<_> = failures
        .into_iter()
        .filter(|failure| !known.iter().any(|prefix| failure.starts_with(prefix)))
        .collect();
    assert!(
        unexpected.is_empty(),
        "rule documentation snippets must execute as documented (see #2416 for known gaps):\n{}",
        unexpected.join("\n")
    );
}
