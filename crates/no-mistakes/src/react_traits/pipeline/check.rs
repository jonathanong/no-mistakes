use crate::react_traits::report::types::{FileConfig, RootConfig, Violation};
use anyhow::Result;
use std::path::Path;

/// Parsed React check settings that can be shared across one request.
#[doc(hidden)]
pub struct PreparedReactCheck {
    file_config: FileConfig,
    effective_no_fetch: bool,
}

impl PreparedReactCheck {
    pub fn enabled(&self) -> bool {
        self.effective_no_fetch
    }
}

pub fn run_check(
    root: &Path,
    config_path: Option<&Path>,
    targets: &[String],
    assert_no_fetch: bool,
) -> Result<Vec<Violation>> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible_paths = snapshot.paths_for(root);
    let stems = [".no-mistakes"];
    let root_config: RootConfig =
        crate::config::load_config_from_visible(root, config_path, &stems, &visible_paths)?;
    let prepared = prepare(root_config, assert_no_fetch);
    if !prepared.enabled() {
        return Ok(Vec::new());
    }
    let facts_list = crate::react_traits::pipeline::run::run_analyze_inner_from_visible(
        root,
        &prepared.file_config,
        targets,
        None,
        &visible_paths,
    )?;
    Ok(assert_no_fetch_violations(&facts_list))
}

pub fn run_check_with_facts(
    root: &Path,
    config_path: Option<&Path>,
    targets: &[String],
    assert_no_fetch: bool,
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<Violation>> {
    let stems = [".no-mistakes"];
    let root_config: RootConfig =
        crate::config::load_config_from_visible(root, config_path, &stems, shared.files())?;
    let prepared = prepare(root_config, assert_no_fetch);
    run_check_with_prepared_facts(root, targets, shared, &prepared)
}

/// Prepare aggregate-check React settings from the unified config that the
/// caller has already loaded. Standalone React commands keep their historical
/// config-loading wrappers above.
#[doc(hidden)]
pub fn prepare_check_from_loaded_config(
    config: &crate::config::v2::NoMistakesConfig,
    assert_no_fetch: bool,
) -> PreparedReactCheck {
    prepare_file_config(file_config_from_loaded(config), assert_no_fetch)
}

pub(crate) fn file_config_from_loaded(config: &crate::config::v2::NoMistakesConfig) -> FileConfig {
    let mut file_config = FileConfig {
        frontend_root: config.frontend_root.clone(),
        assert_no_fetch: config.assert_no_fetch,
    };
    if let Some(react_traits) = &config.react_traits {
        if react_traits.frontend_root.is_some() {
            file_config
                .frontend_root
                .clone_from(&react_traits.frontend_root);
        }
        if react_traits.assert_no_fetch.is_some() {
            file_config.assert_no_fetch = react_traits.assert_no_fetch;
        }
    }
    file_config
}

#[doc(hidden)]
pub fn run_check_with_prepared_facts(
    root: &Path,
    targets: &[String],
    shared: &crate::codebase::check_facts::CheckFactMap,
    prepared: &PreparedReactCheck,
) -> Result<Vec<Violation>> {
    if !prepared.enabled() {
        return Ok(Vec::new());
    }
    let facts_list = crate::react_traits::pipeline::run_with_facts::run_analyze_inner_with_facts(
        root,
        &prepared.file_config,
        targets,
        shared,
    )?;
    Ok(assert_no_fetch_violations(&facts_list))
}

fn assert_no_fetch_violations(
    facts_list: &[crate::react_traits::ComponentFacts],
) -> Vec<Violation> {
    let mut violations = Vec::new();
    for facts in facts_list {
        let has_fetch = !facts.fetches.is_empty()
            || facts
                .inherited_from_children
                .as_ref()
                .is_some_and(|agg| agg.has_fetch);
        if has_fetch {
            violations.push(Violation {
                component: facts.name.clone(),
                file: facts.file.clone(),
                rule: "assert-no-fetch".to_string(),
                detail: facts.fetches.first().and_then(|f| f.shape.clone()),
            });
        }
    }
    violations
}

pub fn check_enabled(
    root: &Path,
    config_path: Option<&Path>,
    assert_no_fetch: bool,
) -> Result<bool> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible_paths = snapshot.paths_for(root);
    let stems = [".no-mistakes"];
    let root_config: RootConfig =
        crate::config::load_config_from_visible(root, config_path, &stems, &visible_paths)?;
    Ok(prepare(root_config, assert_no_fetch).enabled())
}

fn prepare(root_config: RootConfig, assert_no_fetch: bool) -> PreparedReactCheck {
    prepare_file_config(root_config.into_file_config(), assert_no_fetch)
}

fn prepare_file_config(file_config: FileConfig, assert_no_fetch: bool) -> PreparedReactCheck {
    let effective_no_fetch = assert_no_fetch || file_config.assert_no_fetch.unwrap_or(false);
    PreparedReactCheck {
        file_config,
        effective_no_fetch,
    }
}

#[cfg(test)]
mod tests;
