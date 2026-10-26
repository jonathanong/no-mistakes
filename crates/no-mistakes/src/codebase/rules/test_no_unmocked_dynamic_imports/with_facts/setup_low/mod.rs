mod aliases;
pub(crate) mod classify;
mod syntax;

use super::super::setup_helper_graph;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, GraphFiles};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::{Effort, NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub(super) const REWRITE: &str = "use a top-level static import, a vi.mock or jest.mock call, or a literal require() in the setup file";

pub(super) struct Outcome {
    pub(super) mocks: HashSet<PathBuf>,
    pub(super) diagnostics: AnalysisDiagnostics,
}

pub(super) struct Policy {
    pub(super) low: bool,
    pub(super) unanalyzable: Unanalyzable,
}

/// The rule checks each test once. Any application that is not low keeps that
/// shared walk at max, so a max application cannot lose findings.
pub(super) fn policy(config: &NoMistakesConfig) -> Result<Policy> {
    let mut saw = false;
    let mut low = true;
    let mut unanalyzable = Unanalyzable::Ignore;
    for rule in config
        .rules
        .iter()
        .filter(|rule| rule.enabled && rule.rule == super::super::RULE_ID)
    {
        saw = true;
        let resolved = config.effective_rule_analysis(rule)?;
        if resolved.effort != Effort::Low {
            low = false;
        }
        unanalyzable = stricter(unanalyzable, resolved.unanalyzable);
    }
    if !saw || !low {
        return Ok(Policy {
            low: false,
            unanalyzable: Unanalyzable::Report,
        });
    }
    Ok(Policy {
        low: true,
        unanalyzable,
    })
}

fn stricter(left: Unanalyzable, right: Unanalyzable) -> Unanalyzable {
    match (left, right) {
        (Unanalyzable::Report, _) | (_, Unanalyzable::Report) => Unanalyzable::Report,
        (Unanalyzable::Advise, _) | (_, Unanalyzable::Advise) => Unanalyzable::Advise,
        _ => Unanalyzable::Ignore,
    }
}

pub(super) struct Group<'a> {
    pub(super) files: &'a [PathBuf],
    pub(super) root: &'a Path,
    pub(super) resolver: &'a crate::codebase::ts_resolver::ScopedImportResolver<'a>,
    pub(super) graph: &'a DepGraph,
    pub(super) graph_files: &'a GraphFiles,
    pub(super) visible_files: &'a crate::fx::PathSet,
    pub(super) shared: &'a CheckFactMap,
    pub(super) unanalyzable: Unanalyzable,
}

pub(super) fn from_group(group: Group<'_>) -> Result<Outcome> {
    let cache = RefCell::new(HashMap::new());
    let mut mocks = HashSet::new();
    let mut reached = Vec::new();
    for setup in group.files {
        let classified = remember(&cache, setup, || load_setup(&group, setup))?;
        mocks.extend(resolved(&classified.accepted_mocks, setup, &group));
        reached.push(setup.clone());
        let mut active = Vec::new();
        mocks.extend(setup_helper_graph::collect(
            group.graph,
            setup,
            group.visible_files,
            &mocks,
            |file| {
                let classified = remember(&cache, file, || load_helper(&group, file))?;
                Ok(resolved(&classified.accepted_mocks, file, &group))
            },
            |file| match remember(&cache, file, || load_helper(&group, file)) {
                Ok(classified) => resolved(&classified.accepted_requires, file, &group),
                Err(_) => HashSet::new(),
            },
            |file| active.push(file.to_path_buf()),
        )?);
        reached.extend(active);
    }
    reached.sort();
    reached.dedup();
    let mut diagnostics = AnalysisDiagnostics::default();
    for file in &reached {
        let classified = ready(&cache, file)?;
        for shape in classified.rejected {
            diagnostics.unanalyzable(group.unanalyzable, finding(group.root, file, &shape));
        }
    }
    Ok(Outcome { mocks, diagnostics })
}

fn finding(root: &Path, file: &Path, shape: &classify::Rejected) -> RuleFinding {
    let rel = crate::codebase::ts_source::relative_slash_path(root, file);
    RuleFinding {
        rule: super::super::RULE_ID.to_string(),
        file: rel.clone(),
        line: shape.line,
        message: format!(
            "{rel}:{}: setup reachability is unanalyzable at effort low; {REWRITE}",
            shape.line
        ),
        import: None,
        target: Some(shape.target.to_string()),
    }
}

fn resolved(specifiers: &[String], file: &Path, group: &Group<'_>) -> HashSet<PathBuf> {
    super::super::resolve_mock_specifiers(specifiers, file, group.resolver, Some(group.graph_files))
}

#[derive(Clone)]
enum Cached {
    Ready(classify::Classified),
    Failed(String),
}

impl Cached {
    fn into_result(self) -> Result<classify::Classified> {
        match self {
            Self::Ready(classified) => Ok(classified),
            Self::Failed(message) => anyhow::bail!("{message}"),
        }
    }
}

fn remember(
    cache: &RefCell<HashMap<PathBuf, Cached>>,
    file: &Path,
    load: impl FnOnce() -> Result<classify::Classified>,
) -> Result<classify::Classified> {
    if let Some(entry) = cache.borrow().get(file) {
        return entry.clone().into_result();
    }
    let entry = match load() {
        Ok(classified) => Cached::Ready(classified),
        Err(error) => Cached::Failed(error.to_string()),
    };
    cache.borrow_mut().insert(file.to_path_buf(), entry.clone());
    entry.into_result()
}

fn ready(cache: &RefCell<HashMap<PathBuf, Cached>>, file: &Path) -> Result<classify::Classified> {
    cache
        .borrow()
        .get(file)
        .cloned()
        .map(Cached::into_result)
        .transpose()?
        .ok_or_else(|| anyhow::anyhow!("missing setup classification for {}", file.display()))
}

fn load_setup(group: &Group<'_>, file: &Path) -> Result<classify::Classified> {
    let Some(facts) = group.shared.ts.get(file) else {
        anyhow::bail!("missing shared facts for {}", file.display());
    };
    if let Some(error) = &facts.parse_error {
        anyhow::bail!("failed to parse {}: {error}", file.display());
    }
    let Some(dynamic) = facts.dynamic_imports.as_ref() else {
        anyhow::bail!("missing dynamic import facts for {}", file.display());
    };
    Ok(dynamic.setup_reachability.clone().into())
}

fn load_helper(group: &Group<'_>, file: &Path) -> Result<classify::Classified> {
    let Some(facts) = group.shared.ts.get(file) else {
        anyhow::bail!(
            "missing shared facts for imported setup helper {}",
            file.display()
        );
    };
    if let Some(error) = &facts.parse_error {
        anyhow::bail!(
            "failed to parse imported setup helper {}: {error}",
            file.display()
        );
    }
    let Some(dynamic) = facts.dynamic_imports.as_ref() else {
        anyhow::bail!(
            "missing dynamic import facts for imported setup helper {}",
            file.display()
        );
    };
    Ok(dynamic.setup_reachability.clone().into())
}

#[cfg(test)]
mod tests;
