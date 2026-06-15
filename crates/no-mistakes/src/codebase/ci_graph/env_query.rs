//! `no-mistakes ci env <VAR>` — find every workflow definition and
//! `${{ env.VAR }}` reference of an environment variable.
//!
//! Definitions are read from structured `env:` blocks at the workflow, job, and
//! step scopes. References are a textual scan of every string scalar for a
//! `${{ … env.VAR … }}` expression, attributed to the nearest enclosing scope.
//! Matching is case-sensitive (Linux runner semantics) and does not resolve
//! computed expressions. Exact line numbers are intentionally omitted — use
//! `rg 'env.VAR' <file>` for those.

use super::model::CiWarning;
use super::{discover_workflow_files, relative_slash};
use crate::config::v2::schema::CiConfig;
use regex::Regex;
use serde::Serialize;
use serde_yaml::Value;
use std::path::Path;

/// Result of an `env` query across the workflow set.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CiEnvReport {
    /// The queried variable name.
    pub variable: String,
    /// Files containing at least one definition or reference, sorted by path.
    pub files: Vec<CiEnvFile>,
    /// Non-fatal load/parse warnings.
    pub warnings: Vec<CiWarning>,
}

/// A workflow file with matching locations.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CiEnvFile {
    /// Repo-relative, slash-normalized path.
    pub path: String,
    /// Matching locations, sorted deterministically.
    pub locations: Vec<CiEnvLocation>,
}

/// A single definition or reference of the variable.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CiEnvLocation {
    /// Whether the variable is defined or referenced here.
    pub kind: EnvLocationKind,
    /// The scope the location lives in.
    pub scope: EnvScope,
    /// Owning job id for `job`/`step` scopes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job: Option<String>,
    /// The defined value (definitions only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Whether a location defines or references the variable.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum EnvLocationKind {
    Definition,
    Reference,
}

/// The structural scope of a location.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum EnvScope {
    Workflow,
    Job,
    Step,
}

/// Analyze all workflows under `ci.workflow_dirs` for `var`.
pub fn analyze_env(root: &Path, ci: &CiConfig, var: &str) -> CiEnvReport {
    let reference_re = reference_regex(var);
    let mut files = Vec::new();
    let mut warnings = Vec::new();

    for path in discover_workflow_files(root, ci) {
        let rel = relative_slash(root, &path);
        // An unreadable discovered file yields empty content (parses to null → no
        // locations); only genuine parse failures warrant a warning.
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let value: Value = match serde_yaml::from_str(&content) {
            Ok(value) => value,
            Err(error) => {
                warnings.push(CiWarning {
                    path: rel,
                    message: format!("could not parse workflow YAML: {error}"),
                });
                continue;
            }
        };
        let locations = collect_locations(&value, var, &reference_re);
        if !locations.is_empty() {
            files.push(CiEnvFile {
                path: rel,
                locations,
            });
        }
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    CiEnvReport {
        variable: var.to_string(),
        files,
        warnings,
    }
}

fn reference_regex(var: &str) -> Regex {
    // `env.` must be a standalone context, not a property segment (e.g.
    // `github.event.inputs.env.FOO`). Require it to sit at the start of the
    // expression (right after `${{`) or be preceded by a non-identifier,
    // non-`.` character, so a leading `.env` does not match.
    let pattern = [
        r"\$\{\{(?:[^}]*[^.\w])?env\.",
        &regex::escape(var),
        r"\b[^}]*\}\}",
    ]
    .concat();
    // The pattern is always valid: escaped var plus fixed structure.
    Regex::new(&pattern).expect("env reference regex is well-formed")
}

fn collect_locations(value: &Value, var: &str, reference_re: &Regex) -> Vec<CiEnvLocation> {
    let mut out = Vec::new();
    let Some(root_map) = value.as_mapping() else {
        return out;
    };

    collect_env_defs(root_map, EnvScope::Workflow, None, var, &mut out);
    for (key, val) in root_map {
        if key.as_str() == Some("jobs") {
            continue;
        }
        scan_refs(val, EnvScope::Workflow, None, reference_re, &mut out);
    }

    if let Some(jobs) = root_map.get("jobs").and_then(Value::as_mapping) {
        for (job_key, job_val) in jobs {
            let job_id = job_key.as_str().unwrap_or_default().to_string();
            let Some(job_map) = job_val.as_mapping() else {
                continue;
            };
            collect_env_defs(job_map, EnvScope::Job, Some(&job_id), var, &mut out);
            for (key, val) in job_map {
                if key.as_str() == Some("steps") {
                    continue;
                }
                scan_refs(val, EnvScope::Job, Some(&job_id), reference_re, &mut out);
            }
            if let Some(steps) = job_map.get("steps").and_then(Value::as_sequence) {
                for step in steps {
                    if let Some(step_map) = step.as_mapping() {
                        collect_env_defs(step_map, EnvScope::Step, Some(&job_id), var, &mut out);
                    }
                    scan_refs(step, EnvScope::Step, Some(&job_id), reference_re, &mut out);
                }
            }
        }
    }

    sort_locations(out)
}

fn collect_env_defs(
    map: &serde_yaml::Mapping,
    scope: EnvScope,
    job: Option<&str>,
    var: &str,
    out: &mut Vec<CiEnvLocation>,
) {
    let Some(env) = map.get("env").and_then(Value::as_mapping) else {
        return;
    };
    for (key, val) in env {
        if key.as_str() == Some(var) {
            out.push(CiEnvLocation {
                kind: EnvLocationKind::Definition,
                scope,
                job: job.map(str::to_string),
                value: scalar_to_string(val),
            });
        }
    }
}

fn scan_refs(
    value: &Value,
    scope: EnvScope,
    job: Option<&str>,
    reference_re: &Regex,
    out: &mut Vec<CiEnvLocation>,
) {
    match value {
        Value::String(s) => {
            if reference_re.is_match(s) {
                out.push(CiEnvLocation {
                    kind: EnvLocationKind::Reference,
                    scope,
                    job: job.map(str::to_string),
                    value: None,
                });
            }
        }
        Value::Sequence(seq) => {
            for item in seq {
                scan_refs(item, scope, job, reference_re, out);
            }
        }
        Value::Mapping(map) => {
            for (_, val) in map {
                scan_refs(val, scope, job, reference_re, out);
            }
        }
        _ => {}
    }
}

fn scalar_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Sort for deterministic output. We do NOT dedupe: two `${{ env.FOO }}`
/// references in different steps of the same job look identical (no line/step
/// id is recorded) but are distinct occurrences the user is auditing for.
fn sort_locations(mut locations: Vec<CiEnvLocation>) -> Vec<CiEnvLocation> {
    locations.sort_by(|a, b| {
        (a.kind, a.scope, &a.job, &a.value).cmp(&(b.kind, b.scope, &b.job, &b.value))
    });
    locations
}

#[cfg(test)]
mod tests;
