use super::scripts::{self, ScriptHit, ScriptHitKind, WorkingDir};
use crate::codebase::ts_source::SourceStore;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const MAX_ACTION_DEPTH: usize = 4;

pub(super) struct CompositeOutcome {
    pub(super) action_path: String,
    pub(super) hits: Vec<ScriptHit>,
}

pub(super) fn scan_local_action(
    root: &Path,
    sources: &SourceStore,
    uses: &str,
    script_dirs: &[String],
) -> Option<CompositeOutcome> {
    let rel_dir = normalize_local_uses(uses)?;
    let mut visited = BTreeSet::new();
    let mut hits = Vec::new();
    let action_path = walk_action(
        root,
        sources,
        &rel_dir,
        script_dirs,
        0,
        &mut visited,
        &mut hits,
    );
    Some(CompositeOutcome { action_path, hits })
}

fn walk_action(
    root: &Path,
    sources: &SourceStore,
    rel_dir: &str,
    script_dirs: &[String],
    depth: usize,
    visited: &mut BTreeSet<String>,
    hits: &mut Vec<ScriptHit>,
) -> String {
    let Some((action_path, full_path, source)) = read_action(root, sources, rel_dir) else {
        let missing = if rel_dir.is_empty() {
            "action.yml".to_string()
        } else {
            format!("{rel_dir}/action.yml")
        };
        hits.push(ScriptHit {
            line: 1,
            kind: ScriptHitKind::Unanalyzable {
                token: missing.clone(),
                detail: format!(
                    "calls local action `{missing}`, which could not be read to check repository scripts against workflow path filters."
                ),
            },
        });
        return missing;
    };
    if visited.contains(&action_path) {
        return action_path;
    }
    if depth >= MAX_ACTION_DEPTH {
        hits.push(unanalyzable_hit(
            &action_path,
            &format!(
                "calls local action `{action_path}` nested more than {MAX_ACTION_DEPTH} levels deep, so its repository scripts could not be checked against workflow path filters."
            ),
        ));
        return action_path;
    }
    visited.insert(action_path.clone());
    let Ok(value) = sources.parse_loaded_yaml(&full_path, &source, true) else {
        hits.push(ScriptHit {
            line: 1,
            kind: ScriptHitKind::Unanalyzable {
                token: action_path.clone(),
                detail: format!(
                    "calls local action `{action_path}` containing invalid YAML, so its repository scripts could not be checked against workflow path filters."
                ),
            },
        });
        return action_path;
    };
    let Some(runs) = value.get("runs").and_then(Value::as_mapping) else {
        return action_path;
    };
    if mapping_get(runs, "using").and_then(Value::as_str) != Some("composite") {
        return action_path;
    }
    let Some(steps) = mapping_get(runs, "steps").and_then(Value::as_sequence) else {
        return action_path;
    };
    for step in steps {
        let Some(step) = step.as_mapping() else {
            continue;
        };
        if let Some(run_val) = mapping_get(step, "run") {
            if let Some(script) = run_val.as_str() {
                let dir = step_working_dir(step);
                hits.extend(scripts::scan_script(script, dir, script_dirs));
            } else {
                hits.push(unanalyzable_hit(
                    "run",
                    "has a non-string `run` value; write a literal shell script so workflow path filters can be checked.",
                ));
            }
        }
        if let Some(nested) = mapping_get(step, "uses").and_then(Value::as_str) {
            if let Some(nested_dir) = normalize_local_uses(nested) {
                walk_action(
                    root,
                    sources,
                    &nested_dir,
                    script_dirs,
                    depth + 1,
                    visited,
                    hits,
                );
            } else if nested.trim().starts_with("./") {
                hits.push(unanalyzable_hit(
                    nested,
                    &format!(
                        "uses dynamic local action `{nested}`, which cannot be statically checked against workflow path filters."
                    ),
                ));
            }
        }
    }
    action_path
}

fn unanalyzable_hit(token: &str, detail: &str) -> ScriptHit {
    ScriptHit {
        line: 1,
        kind: ScriptHitKind::Unanalyzable {
            token: token.to_string(),
            detail: detail.to_string(),
        },
    }
}

fn step_working_dir(step: &Mapping) -> WorkingDir {
    match mapping_get(step, "working-directory") {
        None | Some(Value::Null) => WorkingDir::Known(String::new()),
        Some(value) => super::workflow::parse_working_dir(value),
    }
}

fn read_action(
    root: &Path,
    sources: &SourceStore,
    rel_dir: &str,
) -> Option<(String, PathBuf, String)> {
    for filename in ["action.yml", "action.yaml"] {
        let rel = if rel_dir.is_empty() {
            filename.to_string()
        } else {
            format!("{rel_dir}/{filename}")
        };
        let full = root.join(&rel);
        if let Ok(source) = sources.read_path(&full) {
            return Some((rel, full, source.to_string()));
        }
    }
    None
}

pub(super) fn normalize_local_uses(uses: &str) -> Option<String> {
    let trimmed = uses.trim();
    let rest = trimmed.strip_prefix("./")?;
    if rest.contains("${{") {
        return None;
    }
    scripts::resolve_rel_path("", rest)
}

fn mapping_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_owned()))
}
