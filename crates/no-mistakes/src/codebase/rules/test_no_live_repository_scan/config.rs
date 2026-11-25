use crate::config::v2::schema::StringOrList;
use anyhow::{bail, Result};
use serde::Deserialize;

use super::RULE_ID;
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::config::v2::NoMistakesConfig;

pub(crate) fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    config.rule_configured(RULE_ID).then(|| {
        let mut plan = GraphBuildPlan::imports_and_workspace();
        plan.calls = true;
        plan.value_import_selectors = true;
        plan
    })
}

/// True when an application selects Vitest roots, so the catalog must be prepared.
pub fn vitest_roots_configured(config: &NoMistakesConfig) -> bool {
    config.rule_applications(RULE_ID).iter().any(|application| {
        application
            .options
            .get("roots")
            .and_then(serde_yaml::Value::as_sequence)
            .is_none_or(|roots| roots.iter().any(|root| root.get("vitest").is_some()))
    })
}

/// Segments that mark a path as bounded fixture input.
pub(super) const DEFAULT_FIXTURE_DIRS: &[&str] = &["fixtures", "__fixtures__"];

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::codebase::rules) struct Options {
    /// `forbidden-calls` root selectors. Omitted selects every Vitest project.
    pub(super) roots: Option<serde_yaml::Value>,
    pub(super) exclude_roots: Option<StringOrList>,
    /// Path segments that mark bounded fixture input. Replaces the defaults.
    pub(super) fixture_dirs: Option<Vec<String>>,
    /// Extra enumerating imports, in addition to the built-in `fs` and glob set.
    pub(super) enumerators: Vec<Enumerator>,
    pub(super) unresolved_root: UnresolvedRoot,
    /// Shipped asset trees that production code legitimately reads at runtime.
    pub(super) runtime_assets: Vec<RuntimeAsset>,
    /// Analyzer commands tests must not run against the live repository.
    pub(super) analyzers: Vec<AnalyzerCommand>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct RuntimeAsset {
    /// Repository-relative directory; a read at or under it is a runtime asset read.
    pub(super) root: String,
    /// Why the tree is shipped input rather than a policy surface.
    pub(super) reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct AnalyzerCommand {
    /// Executable name. A path matches by its basename, including `node_modules/.bin`.
    pub(super) command: String,
    /// Leading arguments that identify the analyzer. Extra arguments may follow.
    #[serde(default)]
    pub(super) args: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Enumerator {
    pub(super) module: String,
    /// Omitted matches every export of `module`.
    pub(super) export: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum UnresolvedRoot {
    #[default]
    Advise,
    Report,
}

impl Options {
    pub(super) fn fixture_dirs(&self) -> Vec<&str> {
        self.fixture_dirs.as_ref().map_or_else(
            || DEFAULT_FIXTURE_DIRS.to_vec(),
            |dirs| dirs.iter().map(String::as_str).collect(),
        )
    }

    pub(in crate::codebase::rules) fn validate(&self) -> Result<()> {
        let empty = |value: &str| value.trim().is_empty() || value.contains(['/', '\\']);
        if let Some(dirs) = &self.fixture_dirs {
            if dirs.is_empty() || dirs.iter().any(|dir| empty(dir)) {
                bail!("{RULE_ID}: fixtureDirs must list path segments without separators");
            }
        }
        if self.enumerators.iter().any(|e| {
            e.module.trim().is_empty() || e.export.as_deref().is_some_and(|x| x.trim().is_empty())
        }) {
            bail!("{RULE_ID}: enumerators require a non-empty module and export");
        }
        for asset in &self.runtime_assets {
            let segments = || {
                asset
                    .root
                    .split('/')
                    .filter(|part| !part.is_empty() && *part != ".")
            };
            if asset.reason.trim().is_empty()
                || asset.root.contains('\\')
                || asset.root.starts_with('/')
                || segments().next().is_none()
                || segments().any(|part| part == "..")
            {
                bail!("{RULE_ID}: runtimeAssets entries need a relative `root` without `..` and a non-empty `reason`");
            }
        }
        let command_name = |command: &str| {
            !command.is_empty()
                && !command.contains(['/', '\\', ' '])
                && command.chars().all(|c| c.is_ascii_graphic())
        };
        if self.analyzers.iter().any(|analyzer| {
            !command_name(&analyzer.command) || analyzer.args.iter().any(String::is_empty)
        }) {
            bail!("{RULE_ID}: analyzers entries need a command name without separators and non-empty leading arguments");
        }
        Ok(())
    }
}
