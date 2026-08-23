use crate::codebase::lang_frontends::{
    collect_all_lang_facts, lang_config_from_v2, lang_config_is_empty, LangFileFacts,
};
use crate::codebase::test_filter::TestFileFilter;
use crate::config::v2::ConfigView;
use crate::server_routes::graph::{build_filter, PreparedServerAnalysis};
use crate::server_routes::model::{FileFacts, RouteSite};
use crate::server_routes::types::Framework;
use globset::GlobSet;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn merge_language_route_facts(
    prepared: &PreparedServerAnalysis,
    facts: &mut HashMap<PathBuf, FileFacts>,
    cli_filter: Option<&GlobSet>,
) {
    let Some(config) = prepared.config.as_ref() else {
        return;
    };
    merge_configured_lang_routes(prepared, facts, cli_filter, config);
    merge_dotnet_routes(prepared, facts, cli_filter, config);
}

fn merge_configured_lang_routes(
    prepared: &PreparedServerAnalysis,
    facts: &mut HashMap<PathBuf, FileFacts>,
    cli_filter: Option<&GlobSet>,
    config: &crate::config::v2::NoMistakesConfig,
) {
    let lang = lang_config_from_v2(config);
    if lang_config_is_empty(&lang) {
        return;
    }
    let dataset = prepared.session.dataset(&prepared.root);
    let all_files = dataset.paths_for(&prepared.root);
    let sources = dataset.sources_for(&prepared.root);
    let collected = collect_all_lang_facts(&prepared.root, &all_files, &lang, &sources);
    let config_route_filter = build_filter(&ConfigView::new(config).server_route_globs())
        .ok()
        .flatten();
    let test_filter = TestFileFilter::new(&prepared.root, config);
    for map in crate::codebase::lang_frontends::each_lang_map(&collected) {
        for file in map.files.values() {
            merge_file_routes(
                &prepared.root,
                file,
                facts,
                cli_filter,
                config_route_filter.as_ref(),
                Some(&test_filter),
            );
        }
    }
}

fn merge_dotnet_routes(
    prepared: &PreparedServerAnalysis,
    facts: &mut HashMap<PathBuf, FileFacts>,
    cli_filter: Option<&GlobSet>,
    config: &crate::config::v2::NoMistakesConfig,
) {
    let projects =
        crate::codebase::dotnet::configured_projects(&prepared.root, &config.tests.dotnet);
    if projects.is_empty() {
        return;
    }
    let dataset = prepared.session.dataset(&prepared.root);
    let all_files = dataset.paths_for(&prepared.root);
    let sources = dataset.sources_for(&prepared.root);
    let collected = crate::codebase::dotnet::collect_dotnet_facts_with_sources(
        &prepared.root,
        &all_files,
        &projects,
        Some(&sources),
    );
    let config_route_filter = build_filter(&ConfigView::new(config).server_route_globs())
        .ok()
        .flatten();
    let test_filter = TestFileFilter::new(&prepared.root, config);
    for file in collected.files.values() {
        let lang_file = LangFileFacts {
            path: file.path.clone(),
            route_handlers: file.route_handlers.clone(),
            ..Default::default()
        };
        merge_file_routes(
            &prepared.root,
            &lang_file,
            facts,
            cli_filter,
            config_route_filter.as_ref(),
            Some(&test_filter),
        );
    }
}

fn merge_file_routes(
    root: &Path,
    file: &LangFileFacts,
    facts: &mut HashMap<PathBuf, FileFacts>,
    cli_filter: Option<&GlobSet>,
    config_route_filter: Option<&GlobSet>,
    test_filter: Option<&TestFileFilter>,
) {
    if file.route_handlers.is_empty() {
        return;
    }
    let rel = file.path.strip_prefix(root).unwrap_or(&file.path);
    let matches_config = config_route_filter
        .map(|filter| filter.is_match(rel))
        .unwrap_or(true);
    let matches_cli = cli_filter
        .map(|filter| filter.is_match(rel))
        .unwrap_or(true);
    let is_test = test_filter.is_some_and(|filter| filter.is_match(root, &file.path));
    if !(matches_config && matches_cli && !is_test) {
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
        });
    facts
        .entry(file.path.clone())
        .or_default()
        .routes
        .extend(sites);
}

#[cfg(test)]
#[path = "lang_tests.rs"]
mod tests;
