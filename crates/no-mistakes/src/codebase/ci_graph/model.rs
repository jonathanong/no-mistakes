//! Typed model of a GitHub Actions workflow used by the `ci` analyses.
//!
//! The model intentionally captures only what the impact and permission
//! analyses need: trigger path filters, workflow/job permissions, and the job
//! list. Env-variable analysis walks the raw YAML separately (see
//! [`super::env_query`]).

use serde::Serialize;
use std::collections::BTreeMap;

/// A parsed GitHub Actions workflow file.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Workflow {
    /// Repo-relative, slash-normalized path.
    pub path: String,
    /// Top-level `name:`.
    pub name: Option<String>,
    /// Per-event path filters from `on:`.
    pub triggers: Triggers,
    /// Workflow-level default `permissions:`.
    pub permissions: PermissionSpec,
    /// Jobs declared under `jobs:`.
    pub jobs: Vec<Job>,
    /// True when `on:` includes `workflow_call:` (a reusable workflow).
    pub is_reusable: bool,
    /// Non-fatal parse notes (e.g. both `paths` and `paths-ignore` present).
    pub warnings: Vec<String>,
}

/// Trigger configuration relevant to path-based impact analysis.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct Triggers {
    /// Path filters keyed by event name (`push`, `pull_request`,
    /// `pull_request_target`). Only path-filterable events appear here.
    pub events: BTreeMap<String, PathFilter>,
    /// Other events present in `on:` that don't filter on paths
    /// (`workflow_dispatch`, `schedule`, `workflow_call`, …), sorted.
    pub other_events: Vec<String>,
}

/// `paths:` / `paths-ignore:` globs for a single event.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct PathFilter {
    /// `paths:` include globs (may contain `!` negations).
    pub paths: Vec<String>,
    /// `paths-ignore:` exclude globs.
    pub paths_ignore: Vec<String>,
}

impl PathFilter {
    /// True when neither `paths` nor `paths-ignore` constrains the event, so it
    /// runs on any file change.
    pub fn is_unconstrained(&self) -> bool {
        self.paths.is_empty() && self.paths_ignore.is_empty()
    }
}

/// A single job under `jobs:`.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Job {
    /// Job id (the `jobs:` map key).
    pub id: String,
    /// `name:` if present.
    pub name: Option<String>,
    /// Job-level `permissions:`. `Unspecified` inherits the workflow default.
    pub permissions: PermissionSpec,
    /// `uses: ./.github/workflows/x.yml` reusable-workflow call, if any.
    pub uses: Option<String>,
}

/// GitHub Actions `permissions:` value.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "scopes")]
pub enum PermissionSpec {
    /// `permissions:` absent at this level.
    Unspecified,
    /// `permissions: read-all`.
    ReadAll,
    /// `permissions: write-all`.
    WriteAll,
    /// `permissions: {}` — explicitly no permissions.
    Empty,
    /// Explicit `scope: level` map.
    Map(BTreeMap<String, PermissionLevel>),
}

/// Access level for a permission scope.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionLevel {
    Read,
    Write,
    None,
}

/// The GitHub Actions permission scopes that `read-all`/`write-all` expand.
/// Per-scope `read`/`write` capability is applied in
/// [`super::permissions`] so the shorthands never report an impossible level.
pub const PERMISSION_SCOPES: &[&str] = &[
    "actions",
    "attestations",
    "checks",
    "contents",
    "deployments",
    "discussions",
    "id-token",
    "issues",
    "models",
    "packages",
    "pages",
    "pull-requests",
    "repository-projects",
    "security-events",
    "statuses",
    "vulnerability-alerts",
];

/// A non-fatal problem encountered while loading workflows.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CiWarning {
    /// Workflow path the warning relates to (repo-relative, slash-normalized).
    pub path: String,
    /// Human-readable description.
    pub message: String,
}
