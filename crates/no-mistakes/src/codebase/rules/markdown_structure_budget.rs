use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::{Context, Result};
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

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    facts: &super::markdown_facts::MarkdownFactMap,
) -> Result<Vec<RuleFinding>> {
    let markdown = super::markdown_scope::markdown_files(files);
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options()?;
        let max_lines = opts.max_lines.unwrap_or(DEFAULT_MAX_LINES);
        let max_chars = opts.max_chars.unwrap_or(DEFAULT_MAX_CHARS);
        let max_tables = opts.max_tables.unwrap_or(DEFAULT_MAX_TABLES);
        let max_mermaid = opts.max_mermaid.unwrap_or(DEFAULT_MAX_MERMAID);
        let targets =
            super::path_filter::filter_markdown_rule_files(root, config, rule, &markdown)?;
        let scope_roots = super::markdown_scope::scope_roots(root, config, rule);
        let baseline = read_baseline(root, opts.baseline_file.as_deref(), files)?;
        let mut seen = BTreeSet::new();
        for path in targets {
            let Some(_scope_root) = super::markdown_scope::scope_root_for_path(&scope_roots, &path)
            else {
                anyhow::bail!(
                    "{RULE_ID} filtered target {} is outside its configured scope",
                    path.display()
                );
            };
            let markdown = facts.get_for_rule(&path, RULE_ID)?;
            let baseline_key = super::markdown_scope::baseline_key(root, _scope_root, &path);
            let file = super::markdown_scope::finding_key(root, &path);
            let tables = markdown.table_count;
            let mermaid = markdown.mermaid_count;
            let oversized = markdown.line_count > max_lines || markdown.char_count > max_chars;
            let current = BaselineEntry { tables, mermaid };
            if !seen.insert(baseline_key.clone()) {
                anyhow::bail!(
                    "{RULE_ID} has ambiguous baseline key `{baseline_key}` across configured project roots; configure separate rule applications"
                );
            }
            let violates = oversized && (tables > max_tables || mermaid > max_mermaid);
            match (violates, baseline.get(&baseline_key)) {
                (false, Some(_)) => findings.push(stale(&file, "is no longer a structure-budget violation")),
                (false, None) => {},
                (true, Some(entry)) if *entry == current => {},
                (true, Some(_)) => findings.push(stale(&file, "visual counts no longer match the baseline")),
                (true, None) => findings.push(RuleFinding { rule: RULE_ID.to_string(), file, line: 1, message: format!("oversized Markdown has {tables} tables (max {max_tables}) and {mermaid} Mermaid blocks (max {max_mermaid})"), import: None, target: None }),
            }
        }
        for file in baseline.keys() {
            if !seen.contains(file) {
                let finding_file =
                    super::markdown_scope::baseline_finding_key(root, &scope_roots, file, RULE_ID)?;
                findings.push(stale(
                    &finding_file,
                    "references a deleted or excluded Markdown file",
                ));
            }
        }
    }
    super::sort_findings(&mut findings);
    Ok(findings)
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
#[path = "markdown_structure_budget/tests.rs"]
mod tests;
