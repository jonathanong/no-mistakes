use super::github_actions_workflow_files::{
    env_policy::{
        lines::{locate, parse, Seg},
        scalar_text, visit_workflow_scopes,
    },
    gha_kind,
};
use super::RuleFinding;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "github-actions-no-insta-snapshot-updates";
const NAME: &str = "INSTA_UPDATE";

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Options {}

super::sourced_file_rule_entrypoints!();

fn scan(
    root: &Path,
    _options: &Options,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    Ok(files
        .par_iter()
        .filter_map(|path| check_workflow(root, path, sources))
        .flatten()
        .collect())
}

fn check_workflow(root: &Path, path: &Path, sources: &SourceStore) -> Option<Vec<RuleFinding>> {
    let relative = relative_slash_path(root, path);
    if gha_kind(&relative) != Some(false) {
        return None;
    }
    let yaml = sources.parse_yaml_path(path).ok()?;
    let source = super::read_language_source(sources, path)?;
    let locations = parse(&source);
    let mut findings = Vec::new();
    visit_workflow_scopes(&yaml, &mut |owner, path, scope| {
        let Some(mode) = owner
            .get("env")
            .and_then(|env| env.get(NAME))
            .and_then(scalar_text)
            .filter(|mode| matches!(mode.as_str(), "always" | "1" | "force" | "unseen"))
        else {
            return;
        };
        let mut at = path.to_vec();
        at.extend([Seg::Key("env".into()), Seg::Key(NAME.into())]);
        findings.push(RuleFinding {
            rule: RULE_ID.into(),
            file: relative.clone(),
            line: locate(locations.as_ref(), &at),
            message: format!(
                "{scope} sets `{NAME}` to `{mode}`, which can accept snapshots during CI. Set `{NAME}` to `\"no\"` in CI. Review and commit deliberate snapshot changes separately."
            ),
            import: None,
            target: Some(NAME.into()),
            causes: Vec::new(),
        });
    });
    Some(findings)
}
