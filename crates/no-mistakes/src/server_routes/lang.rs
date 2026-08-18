use crate::codebase::lang_frontends::{
    collect_all_lang_facts, lang_config_from_v2, lang_config_is_empty, LangFileFacts,
};
use crate::server_routes::model::{FileFacts, RouteSite};
use crate::server_routes::types::Framework;
use globset::GlobSet;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::graph::PreparedServerAnalysis;

pub(super) fn merge_language_route_facts(
    prepared: &PreparedServerAnalysis,
    facts: &mut HashMap<PathBuf, FileFacts>,
    cli_filter: Option<&GlobSet>,
) {
    let Some(config) = prepared.config.as_ref() else {
        return;
    };
    let lang = lang_config_from_v2(config);
    if lang_config_is_empty(&lang) {
        return;
    }
    let dataset = prepared.session.dataset(&prepared.root);
    let all_files = dataset.paths_for(&prepared.root);
    let sources = dataset.sources_for(&prepared.root);
    let collected = collect_all_lang_facts(&prepared.root, &all_files, &lang, &sources);
    for map in crate::codebase::lang_frontends::each_lang_map(&collected) {
        for file in map.files.values() {
            merge_file_routes(&prepared.root, file, facts, cli_filter);
        }
    }
}

fn merge_file_routes(
    root: &Path,
    file: &LangFileFacts,
    facts: &mut HashMap<PathBuf, FileFacts>,
    cli_filter: Option<&GlobSet>,
) {
    if file.route_handlers.is_empty() {
        return;
    }
    let rel = file.path.strip_prefix(root).unwrap_or(&file.path);
    if cli_filter.is_some_and(|filter| !filter.is_match(rel)) {
        return;
    }
    let sites = file
        .route_handlers
        .iter()
        .map(|(raw_path, _handler)| RouteSite {
            file: file.path.clone(),
            line: 0,
            binding: String::new(),
            method: "*".to_string(),
            raw_path: raw_path.clone(),
            path: raw_path.clone(),
            query_params: Vec::new(),
            framework: Framework::Heuristic,
        })
        .collect::<Vec<_>>();
    if sites.is_empty() {
        return;
    }
    facts
        .entry(file.path.clone())
        .or_default()
        .routes
        .extend(sites);
}

#[cfg(test)]
#[path = "lang_tests.rs"]
mod tests;
