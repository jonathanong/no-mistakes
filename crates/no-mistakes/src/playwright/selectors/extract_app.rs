use super::dynamic_values::DynamicIdentifierValues;
use super::jsx_resolve::{
    app_selector_values, app_selector_values_from_visible, SelectorFileContext,
};
use super::scoped_defaults::{
    collect_scoped_static_identifier_defaults, ScopedStaticIdentifierDefault,
};
use super::types::{AppSelector, SelectorRegexes};
use super::HTML_ID_ATTRIBUTE;
use crate::playwright::ast;
use oxc_ast_visit::Visit;
use oxc_span::GetSpan;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use rayon::prelude::*;
use std::collections::BTreeSet;

mod discovery;

pub fn collect_app_selectors(
    frontend_root: &Path,
    attributes: &[String],
) -> Result<Vec<AppSelector>> {
    collect_app_selectors_with_sources(frontend_root, attributes, None)
}

pub(crate) fn collect_app_selectors_with_sources(
    frontend_root: &Path,
    attributes: &[String],
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<AppSelector>> {
    use super::is_source_file;
    let component_attributes = BTreeMap::new();
    if !frontend_root.exists() {
        return Ok(Vec::new());
    }
    let candidates = discovery::source_file_candidates(frontend_root);
    let visible_files = candidates
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<crate::fx::PathSet>();
    let regexes = super::regex_mod::compile_selector_regexes(attributes, &component_attributes);

    let selectors: BTreeSet<AppSelector> = candidates
        .into_par_iter()
        .filter_map(|path| {
            if !is_source_file(&path) {
                return None;
            }
            Some(
                crate::codebase::ts_source::SourceStore::read_prepared_or_open(sources, &path)
                    .map_err(|error| anyhow::anyhow!("{error}"))
                    .and_then(|source| {
                        extract_app_selectors_with_regexes_from_visible(
                            &path,
                            &source,
                            &regexes,
                            &visible_files,
                            sources,
                        )
                    }),
            )
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();
    Ok(selectors.into_iter().collect())
}

pub fn extract_app_selectors(
    path: &Path,
    source: &str,
    attributes: &[String],
    component_attributes: &BTreeMap<String, String>,
) -> Result<Vec<AppSelector>> {
    use super::regex_mod::compile_selector_regexes;
    let regexes = compile_selector_regexes(attributes, component_attributes);
    extract_app_selectors_with_regexes(path, source, &regexes)
}

pub fn extract_app_selectors_with_regexes(
    path: &Path,
    source: &str,
    regexes: &SelectorRegexes,
) -> anyhow::Result<Vec<AppSelector>> {
    extract_app_selectors_with_regexes_inner(path, source, regexes, None, None)
}

pub(crate) fn extract_app_selectors_with_regexes_from_visible(
    path: &Path,
    source: &str,
    regexes: &SelectorRegexes,
    visible_files: &crate::fx::PathSet,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> anyhow::Result<Vec<AppSelector>> {
    extract_app_selectors_with_regexes_inner(path, source, regexes, Some(visible_files), sources)
}

fn extract_app_selectors_with_regexes_inner(
    path: &Path,
    source: &str,
    regexes: &SelectorRegexes,
    visible_files: Option<&crate::fx::PathSet>,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> anyhow::Result<Vec<AppSelector>> {
    ast::with_program(path, source, |program, source| {
        extract_app_selectors_from_program(
            path,
            source,
            program,
            regexes,
            ExtractProgramOptions {
                visible_files,
                defer_cross_file: false,
                sources,
                recover_dynamic: true,
            },
        )
    })
}

struct ExtractProgramOptions<'a> {
    visible_files: Option<&'a crate::fx::PathSet>,
    defer_cross_file: bool,
    sources: Option<&'a crate::codebase::ts_source::SourceStore>,
    recover_dynamic: bool,
}

pub(crate) fn extract_app_selectors_from_program_from_visible_deferred(
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    regexes: &SelectorRegexes,
    visible_files: &crate::fx::PathSet,
    recover_dynamic: bool,
) -> Vec<AppSelector> {
    extract_app_selectors_from_program(
        path,
        source,
        program,
        regexes,
        ExtractProgramOptions {
            visible_files: Some(visible_files),
            defer_cross_file: true,
            sources: None,
            recover_dynamic,
        },
    )
}

fn extract_app_selectors_from_program(
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    regexes: &SelectorRegexes,
    options: ExtractProgramOptions<'_>,
) -> Vec<AppSelector> {
    let scoped_static_identifier_defaults = collect_scoped_static_identifier_defaults(program);
    let dynamic_identifier_values = if options.recover_dynamic {
        match (options.visible_files, options.defer_cross_file) {
            (Some(visible), true) => {
                super::dynamic_values::collect_dynamic_identifier_values_with_file_from_visible_deferred(
                    program, source, path, visible,
                )
            }
            (Some(visible), false) => {
                super::dynamic_values::collect_dynamic_identifier_values_with_file_from_visible(
                    program, source, path, visible, options.sources,
                )
            }
            (None, _) => super::dynamic_values::collect_dynamic_identifier_values_with_file(
                program, source, path,
            ),
        }
    } else {
        Vec::new()
    };
    let mut visitor = AppSelectorVisitor {
        path,
        source,
        attributes: &regexes.app_attributes,
        component_attributes: &regexes.component_attributes,
        html_ids: regexes.html_ids,
        scoped_static_identifier_defaults: &scoped_static_identifier_defaults,
        dynamic_identifier_values: &dynamic_identifier_values,
        program,
        visible_files: options.visible_files,
        sources: options.sources,
        recover_dynamic: options.recover_dynamic,
        selectors: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.selectors
}

include!("extract_app_visitor.rs");

#[cfg(test)]
mod tests;
