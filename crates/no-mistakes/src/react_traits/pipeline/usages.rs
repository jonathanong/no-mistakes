//! Reverse JSX usage query: given a component file (optionally `path#Symbol`),
//! find every JSX callsite that renders it, the stories/tests that import it, and
//! the prop type names it exports.

mod helpers;

use crate::ast;
use crate::imports::relative_string;
use crate::react_traits::analyze::import_table::build_import_table_from_visible;
use crate::react_traits::analyze::jsx_callsites::collect_jsx_callsites;
use crate::react_traits::pipeline::run::discover_react_files_from_visible;
use crate::react_traits::report::types::{Callsite, RootConfig, UsagesReport, UsagesTarget};
use anyhow::Result;
use helpers::{
    callsite_symbol_matches, filter_importers, importer_symbol_matches, is_story, is_test,
    prop_type_names, same_path, split_target,
};
use rayon::prelude::*;
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

pub use helpers::UsagesInclude;

struct FileHit {
    callsites: Vec<Callsite>,
    importer: Option<String>,
}

#[derive(Clone)]
pub(crate) struct UsageFileFacts {
    imports: Vec<(PathBuf, String)>,
    callsites: Vec<crate::react_traits::analyze::jsx_callsites::RawCallsite>,
}

pub(crate) fn collect_usage_file_facts(
    file: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    visible_files: Option<&HashSet<PathBuf>>,
) -> UsageFileFacts {
    let import_table = match visible_files {
        Some(visible) => build_import_table_from_visible(file, program, visible),
        None => crate::react_traits::analyze::import_table::build_import_table(file, program),
    };
    let imports = import_table
        .values()
        .map(|entry| (entry.resolved_path.clone(), entry.exported_name.clone()))
        .collect();
    let callsites = collect_jsx_callsites(program, &import_table, &file.to_path_buf(), source);
    UsageFileFacts { imports, callsites }
}

pub(crate) fn run_usages_with_loaded_config_and_facts(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    target: &str,
    scan_targets: &[String],
    include: &UsagesInclude,
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<UsagesReport> {
    let root = crate::codebase::ts_source::normalize_discovery_path(root);
    let file_config = super::check::file_config_from_loaded(config);
    let (path_part, symbol) = split_target(target);
    let candidate = if Path::new(path_part).is_absolute() {
        PathBuf::from(path_part)
    } else {
        root.join(path_part)
    };
    if !candidate.exists() {
        anyhow::bail!("target file not found: {}", candidate.display());
    }
    let target_abs = candidate.canonicalize()?;
    let files =
        discover_react_files_from_visible(&root, &file_config, scan_targets, shared.files())?;
    let mut callsites = Vec::new();
    let mut importer_files = BTreeSet::new();
    for file in files {
        let Some(facts) = shared
            .ts
            .get(&file)
            .and_then(|facts| facts.react_usages.as_ref())
        else {
            continue;
        };
        if facts.imports.iter().any(|(path, exported)| {
            same_path(path, &target_abs) && importer_symbol_matches(exported, symbol.as_deref())
        }) {
            importer_files.insert(relative_string(&root, &file));
        }
        callsites.extend(
            facts
                .callsites
                .iter()
                .filter(|callsite| {
                    same_path(&callsite.resolved_path, &target_abs)
                        && callsite_symbol_matches(&callsite.exported_name, symbol.as_deref())
                })
                .map(|callsite| Callsite {
                    file: relative_string(&root, &file),
                    line: callsite.line,
                    component: callsite.exported_name.clone(),
                    props: callsite.props.clone(),
                    has_spread: callsite.has_spread,
                }),
        );
    }
    callsites.sort_by(|a, b| (a.file.as_str(), a.line).cmp(&(b.file.as_str(), b.line)));
    let prop_types = include.prop_types.then(|| {
        shared
            .ts
            .get(&crate::codebase::ts_resolver::normalize_path(&candidate))
            .and_then(|facts| facts.symbols.as_ref())
            .map(helpers::prop_type_names_from_symbols)
            .unwrap_or_default()
    });
    Ok(UsagesReport {
        target: UsagesTarget {
            file: relative_string(&root, &candidate),
            symbol,
        },
        callsites,
        stories: include
            .stories
            .then(|| filter_importers(&importer_files, is_story)),
        tests: include
            .tests
            .then(|| filter_importers(&importer_files, is_test)),
        prop_types,
    })
}

pub fn run_usages(
    root: &Path,
    config_path: Option<&Path>,
    target: &str,
    scan_targets: &[String],
    include: &UsagesInclude,
) -> Result<UsagesReport> {
    let root = crate::codebase::ts_source::normalize_discovery_path(root);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible_paths = snapshot.paths_for(&root);
    let root_config: RootConfig = crate::config::load_config_from_visible(
        &root,
        config_path,
        &[".no-mistakes"],
        &visible_paths,
    )?;
    let file_config = root_config.into_file_config();
    run_usages_from_visible(
        &root,
        target,
        scan_targets,
        include,
        &file_config,
        &visible_paths,
    )
}

include!("usages_scan.rs");

#[cfg(test)]
mod tests;
