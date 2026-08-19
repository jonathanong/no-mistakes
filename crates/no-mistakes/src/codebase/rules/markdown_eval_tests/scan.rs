use super::{is_eval_test, CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let mut observed = BTreeSet::new();
    let mut findings = Vec::new();
    for path in files {
        let rel = relative_slash_path(root, path);
        let Some(source) = super::super::read_source(sources, path) else {
            continue;
        };
        if !is_eval_test(&source, opts) {
            continue;
        }
        if opts.allow.contains(&rel) {
            observed.insert(rel);
            continue;
        }
        findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: rel.clone(),
            line: 1,
            message: format!(
                "{rel} reads a markdown file and evals a shell block inside a spawned bash/sh/zsh process. Port content assertions to a spawn-free test, or add this exact relative path to options.allow with a rationale."
            ),
            import: None,
            target: Some(rel),
        });
    }
    for allowed in &opts.allow {
        if observed.contains(allowed) {
            continue;
        }
        findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: allowed.clone(),
            line: 1,
            message: format!(
                "stale markdown-eval-tests allowlist entry `{allowed}`: it is not a matching markdown-eval test in this run"
            ),
            import: None,
            target: Some(allowed.clone()),
        });
    }
    findings
}
