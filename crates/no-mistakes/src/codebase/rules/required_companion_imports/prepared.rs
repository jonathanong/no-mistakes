//! Request-owned companion selections, shared by fact demand and projection.

use super::helpers::{
    build_globset, build_pattern_globset, companion_patterns, render_template, source_extensions,
    source_info,
};
use super::{Options, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use crate::fx::{FxHashMap, FxHashSet};
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[doc(hidden)]
pub struct PreparedCompanionImportPlan {
    root: PathBuf,
    config: NoMistakesConfig,
    pub(super) checks: Vec<SourceCheck>,
    pub(crate) import_paths: Vec<PathBuf>,
}

pub(super) struct SourceCheck {
    pub(super) rel: String,
    pub(super) expected: String,
    pub(super) companions: Arc<Vec<PathBuf>>,
}

struct Candidate {
    file: PathBuf,
    rel: String,
    glob_rels: Vec<String>,
}

type CompanionSelections = FxHashMap<(Vec<PathBuf>, Vec<String>), Arc<Vec<PathBuf>>>;

#[doc(hidden)]
pub fn configure_fact_plan(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    plan: &mut CheckFactPlan,
) -> Result<()> {
    let rules = config.rule_applications(RULE_ID);
    if rules.is_empty() {
        return Ok(());
    }
    let prepared = prepare(root, config, files, sources, &rules)?;
    if prepared
        .import_paths
        .iter()
        .any(|path| !plan.production_import_paths.contains(path))
    {
        Arc::make_mut(&mut plan.production_import_paths)
            .extend(prepared.import_paths.iter().cloned());
    }
    Arc::make_mut(&mut plan.companion_import_plans).push(Arc::new(prepared));
    Ok(())
}

impl PreparedCompanionImportPlan {
    pub(super) fn matches(&self, root: &Path, config: &NoMistakesConfig) -> bool {
        self.root == normalize_path(root) && self.config == *config
    }
}

fn prepare(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    rules: &[&RuleDef],
) -> Result<PreparedCompanionImportPlan> {
    let root = normalize_path(root);
    let skip = super::super::skip_dir_set(config);
    let mut universes: FxHashMap<Vec<PathBuf>, Vec<Candidate>> = FxHashMap::default();
    let mut selections = CompanionSelections::default();
    let mut checks = Vec::new();
    for rule in rules {
        let opts: Options = rule.try_rule_options()?;
        let roots = super::super::target_roots(&root, config, rule);
        let filter = super::super::path_filter::RulePathFilter::new(&root, config, rule)?;
        if opts.companion_globs.is_empty() || opts.specifier_template.is_empty() {
            continue;
        }
        let candidates = universes.entry(roots.clone()).or_insert_with(|| {
            if let Some(observer) = sources.observer() {
                observer.increment("companion_imports.candidate_universes", 1);
            }
            files
                .iter()
                .filter(|file| {
                    super::super::file_allowed_by_roots_and_skip(&root, &skip, file, &roots)
                })
                .map(|file| Candidate {
                    file: normalize_path(file),
                    rel: relative_slash_path(&root, file),
                    glob_rels: relative_paths(&root, file, &roots),
                })
                .collect()
        });
        let globs = build_globset(&opts.source_globs)?;
        let extensions = source_extensions(&opts);
        let excluded: HashSet<&str> = opts.exclude_basenames.iter().map(String::as_str).collect();
        let mut application_checks = Vec::new();
        let mut companion_files = FxHashSet::default();
        for candidate in candidates
            .iter()
            .filter(|candidate| filter.is_match(&candidate.file))
        {
            let Some(source) = source_info(
                &candidate.rel,
                &candidate.glob_rels,
                &opts,
                globs.as_ref(),
                &extensions,
                &excluded,
            ) else {
                continue;
            };
            let patterns = companion_patterns(&opts, &source);
            let mut key_patterns = patterns.clone();
            key_patterns.sort();
            key_patterns.dedup();
            let key = (roots.clone(), key_patterns);
            let companions = match selections.get(&key) {
                Some(companions) => Arc::clone(companions),
                None => {
                    let globset = build_pattern_globset(&patterns)?;
                    if let Some(observer) = sources.observer() {
                        observer.increment("companion_imports.selections", 1);
                    }
                    let matches = Arc::new(
                        candidates
                            .iter()
                            .filter(|candidate| {
                                candidate.glob_rels.iter().any(|rel| globset.is_match(rel))
                            })
                            .map(|candidate| candidate.file.clone())
                            .collect::<Vec<_>>(),
                    );
                    selections.insert(key, Arc::clone(&matches));
                    matches
                }
            };
            companion_files.extend(companions.iter().cloned());
            application_checks.push((
                candidate.file.clone(),
                SourceCheck {
                    rel: source.rel.clone(),
                    expected: render_template(&opts.specifier_template, &source),
                    companions,
                },
            ));
        }
        checks.extend(
            application_checks
                .into_iter()
                .filter(|(file, _)| !companion_files.contains(file))
                .map(|(_, check)| check),
        );
    }
    let mut import_paths = checks
        .iter()
        .flat_map(|check| check.companions.iter().cloned())
        .collect::<Vec<_>>();
    import_paths.sort();
    import_paths.dedup();
    Ok(PreparedCompanionImportPlan {
        root,
        config: config.clone(),
        checks,
        import_paths,
    })
}

fn relative_paths(root: &Path, file: &Path, roots: &[PathBuf]) -> Vec<String> {
    let mut paths = roots
        .iter()
        .filter(|target| file.starts_with(target))
        .map(|target| relative_slash_path(target, file))
        .collect::<Vec<_>>();
    let repo_rel = relative_slash_path(root, file);
    if !paths.contains(&repo_rel) {
        paths.push(repo_rel);
    }
    paths
}
