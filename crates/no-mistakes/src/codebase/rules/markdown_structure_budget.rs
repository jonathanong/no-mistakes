use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::{Context, Result};
use pulldown_cmark::{CodeBlockKind, Event, Options as MarkdownOptions, Parser, Tag};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "markdown-structure-budget";
const DEFAULT_MAX_LINES: usize = 180;
const DEFAULT_MAX_CHARS: usize = 12_000;
const DEFAULT_MAX_TABLES: usize = 1;
const DEFAULT_MAX_MERMAID: usize = 1;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    max_lines: Option<usize>,
    max_chars: Option<usize>,
    max_tables: Option<usize>,
    max_mermaid: Option<usize>,
    baseline_file: Option<PathBuf>,
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
struct BaselineEntry {
    tables: usize,
    mermaid: usize,
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let markdown = files
        .iter()
        .filter(|path| path.starts_with(root) && path.extension().is_some_and(|ext| ext == "md"))
        .cloned()
        .collect::<Vec<_>>();
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let max_lines = opts.max_lines.unwrap_or(DEFAULT_MAX_LINES);
        let max_chars = opts.max_chars.unwrap_or(DEFAULT_MAX_CHARS);
        let max_tables = opts.max_tables.unwrap_or(DEFAULT_MAX_TABLES);
        let max_mermaid = opts.max_mermaid.unwrap_or(DEFAULT_MAX_MERMAID);
        let targets = super::path_filter::filter_rule_files(root, config, rule, &markdown)?;
        let baseline = read_baseline(root, opts.baseline_file.as_deref(), files)?;
        let mut seen = BTreeSet::new();
        for path in targets {
            let Some(content) = super::read_source(sources, &path) else {
                continue;
            };
            let file = relative_slash_path(root, &path);
            let (tables, mermaid) = counts(&content);
            let oversized =
                content.lines().count() > max_lines || content.chars().count() > max_chars;
            let current = BaselineEntry { tables, mermaid };
            seen.insert(file.clone());
            let violates = oversized && (tables > max_tables || mermaid > max_mermaid);
            match (violates, baseline.get(&file)) {
                (false, Some(_)) => findings.push(stale(&file, "is no longer a structure-budget violation")),
                (false, None) => {},
                (true, Some(entry)) if *entry == current => {},
                (true, Some(_)) => findings.push(stale(&file, "visual counts no longer match the baseline")),
                (true, None) => findings.push(RuleFinding { rule: RULE_ID.to_string(), file, line: 1, message: format!("oversized Markdown has {tables} tables (max {max_tables}) and {mermaid} Mermaid blocks (max {max_mermaid})"), import: None, target: None }),
            }
        }
        for file in baseline.keys() {
            if !seen.contains(file) {
                findings.push(stale(
                    file,
                    "references a deleted or excluded Markdown file",
                ));
            }
        }
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}
fn counts(content: &str) -> (usize, usize) {
    let mut tables = 0;
    let mut mermaid = 0;
    for event in Parser::new_ext(content, MarkdownOptions::all()) {
        match event {
            Event::Start(Tag::Table(_)) => tables += 1,
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info)))
                if info
                    .split_whitespace()
                    .next()
                    .is_some_and(|token| token.eq_ignore_ascii_case("mermaid")) =>
            {
                mermaid += 1
            }
            _ => {}
        }
    }
    (tables, mermaid)
}
fn read_baseline(
    root: &Path,
    path: Option<&Path>,
    tracked_files: &[PathBuf],
) -> Result<BTreeMap<String, BaselineEntry>> {
    let Some(path) = path else {
        return Ok(BTreeMap::new());
    };
    let baseline_path = crate::codebase::ts_resolver::normalize_path(&root.join(path));
    let mut baseline_is_tracked = false;
    for file in tracked_files {
        if crate::codebase::ts_resolver::normalize_path(file) == baseline_path {
            baseline_is_tracked = true;
            break;
        }
    }
    if !baseline_is_tracked {
        anyhow::bail!(
            "{RULE_ID} options.baselineFile must reference a tracked repository file: {}",
            path.display()
        )
    }
    let content = std::fs::read_to_string(&baseline_path)
        .context(format!("read {RULE_ID} baseline {}", path.display()))?;
    serde_json::from_str(&content).context("parse markdown-structure-budget baseline JSON")
}
fn stale(file: &str, message: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message: format!("stale baseline entry: {message}"),
        import: None,
        target: None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::v2::{
        schema::{RuleDef, RuleScope},
        NoMistakesConfig,
    };

    fn config(options: &str, include: &[&str], exclude: &[&str]) -> NoMistakesConfig {
        NoMistakesConfig {
            rules: vec![RuleDef {
                rule: RULE_ID.to_string(),
                scope: Some(RuleScope::Repository),
                include: include.iter().map(|item| (*item).to_string()).collect(),
                exclude: exclude.iter().map(|item| (*item).to_string()).collect(),
                options: serde_yaml::from_str(options).unwrap(),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/markdown-structure-budget")
            .join(name)
    }

    fn run(
        root: &Path,
        config: &NoMistakesConfig,
        relative_files: &[&str],
    ) -> Result<Vec<RuleFinding>> {
        let files = relative_files
            .iter()
            .map(|file| root.join(file))
            .collect::<Vec<_>>();
        let sources = super::super::source_store_for_files(&files);
        check_with_files_and_sources(root, config, &files, &sources)
    }
    #[test]
    fn counts_gfm_tables_and_mermaid_fences() {
        assert_eq!(
            counts("| a |\n| --- |\n| b |\n```mermaid\ngraph TD\n```"),
            (1, 1)
        );
        assert_eq!(counts("    ```mermaid\n"), (0, 0));
    }

    #[test]
    fn full_check_uses_strict_thresholds_and_unicode_scalar_characters() {
        let root = fixture(".");
        let crlf = std::fs::read(root.join("exact-crlf.md")).unwrap();
        assert!(crlf.windows(2).any(|pair| pair == b"\r\n"));
        let options = "maxLines: 6\nmaxChars: 100\nmaxTables: 1\nmaxMermaid: 1";
        let findings = run(
            &root,
            &config(options, &["**/*.md"], &[]),
            &["exact-crlf.md", "over-budget.md"],
        )
        .unwrap();
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "over-budget.md");
        let unicode = run(
            &root,
            &config("maxLines: 99\nmaxChars: 3", &["**/*.md"], &[]),
            &["unicode.md"],
        )
        .unwrap();
        assert_eq!(unicode.len(), 1);
    }

    #[test]
    fn counts_multiple_tables_and_case_insensitive_fenced_mermaid_only() {
        let root = fixture(".");
        let findings = run(
            &root,
            &config("maxLines: 1", &["**/*.md"], &[]),
            &["over-budget.md"],
        )
        .unwrap();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("2 tables"));
        assert!(findings[0].message.contains("2 Mermaid"));
    }

    #[test]
    fn baseline_matches_exact_counts_and_stale_cases_fail() {
        let root = fixture("baseline-match");
        let options = "maxLines: 1\nbaselineFile: baseline.json";
        let files = ["doc.md", "baseline.json"];
        assert!(run(&root, &config(options, &["**/*.md"], &[]), &files)
            .unwrap()
            .is_empty());
        let excluded = run(&root, &config(options, &["**/*.md"], &["doc.md"]), &files).unwrap();
        assert_eq!(excluded.len(), 1);
        assert!(excluded[0].message.contains("deleted or excluded"));
    }

    #[test]
    fn baseline_requires_tracked_valid_json() {
        let root = fixture("invalid-baseline");
        let config = config("baselineFile: baseline.json", &["**/*.md"], &[]);
        assert!(run(&root, &config, &["doc.md", "baseline.json"]).is_err());
        assert!(run(&root, &config, &["doc.md"])
            .unwrap_err()
            .to_string()
            .contains("tracked"));
    }

    #[test]
    fn immutable_baseline_variants_detect_resolved_and_changed_counts() {
        for (name, expected) in [
            ("baseline-resolved", "no longer"),
            ("baseline-changed", "no longer match"),
        ] {
            let root = fixture(name);
            let findings = run(
                &root,
                &config(
                    "maxLines: 1\nbaselineFile: baseline.json",
                    &["**/*.md"],
                    &[],
                ),
                &["doc.md", "baseline.json"],
            )
            .unwrap();
            assert_eq!(findings.len(), 1, "{name}: {findings:#?}");
            assert!(findings[0].message.contains(expected));
        }
    }
}
