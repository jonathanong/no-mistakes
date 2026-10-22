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
            for (index, fences) in docs_rule_snippets::snippet_runs(snippet) {
                let fixture = docs_rule_snippets::materialize_snippet(&config, &fences);
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
    let unexpected = unmatched_failures(&failures, &known);
    let stale = unmatched_known_gaps(&failures, &known);
    assert!(
        unexpected.is_empty(),
        "rule documentation snippets must execute as documented (see #2416 for known gaps):\n{}",
        unexpected.join("\n")
    );
    assert!(
        stale.is_empty(),
        "known gap prefixes must still reproduce; remove them once the snippet behaves:\n{}",
        stale.join("\n")
    );
}

fn unmatched_failures(failures: &[String], known: &[&str]) -> Vec<String> {
    failures
        .iter()
        .filter(|failure| !known.iter().any(|prefix| failure.starts_with(prefix)))
        .cloned()
        .collect()
}

fn unmatched_known_gaps<'a>(failures: &[String], known: &[&'a str]) -> Vec<&'a str> {
    known
        .iter()
        .copied()
        .filter(|prefix| !failures.iter().any(|failure| failure.starts_with(*prefix)))
        .collect()
}

#[test]
fn known_gap_prefixes_fail_once_they_stop_reproducing() {
    let failures = ["banned-renamed-files.md: Counterexample[0] expected a finding".to_string()];
    assert_eq!(
        unmatched_known_gaps(
            &failures,
            &[
                "banned-renamed-files.md: Counterexample[0] expected ",
                "gone.md: Valid example[0] expected ",
            ]
        ),
        ["gone.md: Valid example[0] expected "]
    );
    assert_eq!(
        unmatched_failures(
            &failures,
            &["banned-renamed-files.md: Counterexample[0] expected "]
        ),
        [] as [String; 0]
    );
}

#[test]
fn heading_section_skips_hashes_inside_fences_and_accepts_suffixes() {
    let body = "\
## Valid example

```md
## Perf
content
```

## Counterexample and fix

```yaml
bad: true
```
";
    let valid = docs_rule_snippets::heading_section(body, "Valid example");
    assert!(
        valid.contains("## Perf"),
        "heading inside a fence must stay in the Valid example: {valid}"
    );
    let counter = docs_rule_snippets::heading_section(body, "Counterexample");
    assert!(
        counter.contains("bad: true"),
        "Counterexample and fix must match Counterexample: {counter}"
    );
}

#[test]
fn fences_keep_inner_backticks_inside_a_longer_opening() {
    let section = "\
````markdown
```mermaid
flowchart TD
  A --> B
```
````
";
    let extracted = docs_rule_snippets::fences(section);
    assert_eq!(extracted.len(), 1, "{extracted:?}");
    assert_eq!(extracted[0].0, "markdown");
    assert!(extracted[0].1.contains("```mermaid"));
}

#[test]
fn named_file_fences_in_one_section_materialize_together() {
    let runs = docs_rule_snippets::snippet_runs(&[
        ("toml".into(), "# .mise.toml\n[tools]\nfoo = \"1\"\n".into()),
        (
            "yaml".into(),
            "# .github/actions/setup/action.yml\nVERSION: 1\n".into(),
        ),
    ]);
    assert_eq!(runs.len(), 1, "{runs:?}");
    assert_eq!(runs[0].0, 0);
    let fixture = docs_rule_snippets::materialize_snippet(
        "rules:\n  - rule: version-pin-consistency\n    scope: repository\n",
        &runs[0].1,
    );
    assert!(fixture.path().join(".mise.toml").exists());
    assert!(fixture
        .path()
        .join(".github/actions/setup/action.yml")
        .exists());
}

#[test]
fn text_email_is_file_content_not_a_path() {
    let fixture = docs_rule_snippets::materialize_snippet(
        "rules:\n  - rule: test-email-domain-policy\n    scope: repository\n",
        &[("text".into(), "owner@example.com".into())],
    );
    let body = std::fs::read_to_string(fixture.path().join("fixture.txt")).unwrap();
    assert_eq!(
        body, "owner@example.com",
        "a lone email must stay fixture content, not become an empty path"
    );
    assert!(
        !fixture.path().join("owner@example.com").exists(),
        "email addresses are not path listings"
    );
}

#[test]
fn text_at_prefixed_paths_are_path_listings() {
    let fixture = docs_rule_snippets::materialize_snippet(
        "rules:\n  - rule: test-email-domain-policy\n    scope: repository\n",
        &[(
            "text".into(),
            "app/@modal/page.tsx\npackages/@scope/pkg/index.ts".into(),
        )],
    );
    assert!(
        fixture.path().join("app/@modal/page.tsx").exists(),
        "Next.js parallel-route paths must stay named files"
    );
    assert!(
        fixture.path().join("packages/@scope/pkg/index.ts").exists(),
        "scoped package paths must stay named files"
    );
    assert!(
        !fixture.path().join("fixture.txt").exists(),
        "at-prefixed paths are not email content"
    );
}

#[test]
fn composite_action_yaml_is_not_wrapped_as_a_workflow() {
    let fixture = docs_rule_snippets::materialize_snippet(
        "rules:\n  - rule: github-actions-composite-step-schema\n    scope: repository\n",
        &[(
            "yaml".into(),
            "# actions/timeout/action.yml\nname: Timeout Action\nruns:\n  using: composite\n  steps:\n    - run: echo hello\n      shell: bash\n      timeout-minutes: 5\n"
                .into(),
        )],
    );
    let body = std::fs::read_to_string(fixture.path().join("actions/timeout/action.yml")).unwrap();
    assert!(
        body.contains("using: composite"),
        "composite metadata must stay an action file: {body}"
    );
    assert!(
        !body.contains("jobs:"),
        "timeout-minutes on a composite step must not wrap the file as a workflow: {body}"
    );
}

#[test]
fn heading_sections_and_fences_share_delimiter_and_title_boundaries() {
    // Fake headings and malformed closers must remain inside their saved fence.
    let body = read(&repo_root().join("fixtures/docs-coverage/fence-sections/page.md"));
    for (title, expected) in [
        ("Valid example", "## Inside tilde fence"),
        ("Counterexample", "## Inside backtick fence"),
    ] {
        let section = docs_rule_snippets::heading_section(&body, title);
        assert!(section.contains(expected), "{title}: {section}");
        assert!(!section.contains("wrong section"), "{title}: {section}");
        assert!(!section.contains("outside sections"), "{title}: {section}");
        let fences = docs_rule_snippets::fences(&section);
        assert_eq!(fences.len(), 1, "{title}: {fences:?}");
        assert_eq!(fences[0].0, "markdown");
        assert!(fences[0].1.contains(expected), "{title}: {fences:?}");
        assert!(fences[0].1.contains("## Shorter"), "{title}: {fences:?}");
    }
}
