//! Resolve the effective permissions of a job from workflow + job specs.
//!
//! GitHub does not merge job and workflow permissions: a job-level
//! `permissions:` block fully replaces the workflow default. When neither is
//! set, GitHub applies a repository-dependent default token; we report the
//! documented restricted default and flag it as assumed.

use super::model::{Job, PermissionLevel, PermissionSpec, Workflow, PERMISSION_SCOPES};
use serde::Serialize;
use std::collections::BTreeMap;

/// Where a job's effective permissions came from.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionSource {
    /// Job-level `permissions:`.
    Job,
    /// Inherited workflow-level default.
    Workflow,
    /// GitHub's assumed default (no `permissions:` anywhere).
    Default,
}

/// The resolved permissions for a job.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ResolvedPermissions {
    /// Origin of these permissions.
    pub source: PermissionSource,
    /// Scope → access level. Empty means no permissions granted.
    pub scopes: BTreeMap<String, PermissionLevel>,
    /// True when `scopes` is GitHub's assumed default rather than configured.
    pub assumed_default: bool,
}

/// Resolve the effective permissions for `job` within `workflow`.
pub fn effective_permissions(workflow: &Workflow, job: &Job) -> ResolvedPermissions {
    if !matches!(job.permissions, PermissionSpec::Unspecified) {
        return ResolvedPermissions {
            source: PermissionSource::Job,
            scopes: expand(&job.permissions),
            assumed_default: false,
        };
    }
    if !matches!(workflow.permissions, PermissionSpec::Unspecified) {
        return ResolvedPermissions {
            source: PermissionSource::Workflow,
            scopes: expand(&workflow.permissions),
            assumed_default: false,
        };
    }
    ResolvedPermissions {
        source: PermissionSource::Default,
        scopes: assumed_default_scopes(),
        assumed_default: true,
    }
}

/// Expand a spec into an explicit scope → level map.
fn expand(spec: &PermissionSpec) -> BTreeMap<String, PermissionLevel> {
    match spec {
        PermissionSpec::ReadAll => all_scopes(PermissionLevel::Read),
        PermissionSpec::WriteAll => all_scopes(PermissionLevel::Write),
        PermissionSpec::Empty => BTreeMap::new(),
        PermissionSpec::Map(scopes) => scopes.clone(),
        // `Unspecified` is handled by the caller before reaching expand.
        PermissionSpec::Unspecified => BTreeMap::new(),
    }
}

/// Scopes that only support `write`/`none` (never `read`): under `read-all`
/// they are omitted rather than reported with an impossible `read` level.
const WRITE_ONLY_SCOPES: &[&str] = &["id-token"];

/// Scopes that only support `read`/`none` (never `write`): under `write-all`
/// they are capped at `read`, which is what GitHub grants.
const READ_ONLY_SCOPES: &[&str] = &["models", "vulnerability-alerts"];

fn all_scopes(level: PermissionLevel) -> BTreeMap<String, PermissionLevel> {
    PERMISSION_SCOPES
        .iter()
        .filter_map(|scope| {
            if level == PermissionLevel::Read && WRITE_ONLY_SCOPES.contains(scope) {
                return None;
            }
            let effective = if level == PermissionLevel::Write && READ_ONLY_SCOPES.contains(scope) {
                PermissionLevel::Read
            } else {
                level
            };
            Some((scope.to_string(), effective))
        })
        .collect()
}

/// GitHub's documented restricted default token (the exact set depends on
/// repository settings; this is the conservative read-only baseline).
fn assumed_default_scopes() -> BTreeMap<String, PermissionLevel> {
    BTreeMap::from([
        ("contents".to_string(), PermissionLevel::Read),
        ("metadata".to_string(), PermissionLevel::Read),
        ("packages".to_string(), PermissionLevel::Read),
    ])
}

#[cfg(test)]
mod tests;
