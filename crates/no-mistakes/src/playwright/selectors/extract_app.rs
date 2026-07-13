use super::dynamic_values::DynamicIdentifierValues;
use super::jsx_resolve::{app_selector_values, app_selector_values_from_visible};
use super::scoped_defaults::{
    collect_scoped_static_identifier_defaults, ScopedStaticIdentifierDefault,
};
use super::types::{AppSelector, SelectorRegexes};
use super::HTML_ID_ATTRIBUTE;
use crate::playwright::ast;
use oxc_ast_visit::Visit;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use rayon::prelude::*;
use std::collections::BTreeSet;

mod discovery;

pub fn collect_app_selectors(
    frontend_root: &Path,
    attributes: &[String],
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
        .collect::<HashSet<_>>();
    let regexes = super::regex_mod::compile_selector_regexes(attributes, &component_attributes);

    let selectors: BTreeSet<AppSelector> = candidates
        .into_par_iter()
        .filter_map(|path| {
            if !is_source_file(&path) {
                return None;
            }
            Some(
                std::fs::read_to_string(&path)
                    .map_err(|e| e.into())
                    .and_then(|source| {
                        extract_app_selectors_with_regexes_from_visible(
                            &path,
                            &source,
                            &regexes,
                            &visible_files,
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
    extract_app_selectors_with_regexes_inner(path, source, regexes, None)
}

pub(crate) fn extract_app_selectors_with_regexes_from_visible(
    path: &Path,
    source: &str,
    regexes: &SelectorRegexes,
    visible_files: &HashSet<PathBuf>,
) -> anyhow::Result<Vec<AppSelector>> {
    extract_app_selectors_with_regexes_inner(path, source, regexes, Some(visible_files))
}

fn extract_app_selectors_with_regexes_inner(
    path: &Path,
    source: &str,
    regexes: &SelectorRegexes,
    visible_files: Option<&HashSet<PathBuf>>,
) -> anyhow::Result<Vec<AppSelector>> {
    ast::with_program(path, source, |program, source| {
        extract_app_selectors_from_program(path, source, program, regexes, visible_files, false)
    })
}

pub(crate) fn extract_app_selectors_from_program_from_visible_deferred(
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    regexes: &SelectorRegexes,
    visible_files: &HashSet<PathBuf>,
) -> Vec<AppSelector> {
    extract_app_selectors_from_program(path, source, program, regexes, Some(visible_files), true)
}

fn extract_app_selectors_from_program(
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    regexes: &SelectorRegexes,
    visible_files: Option<&HashSet<PathBuf>>,
    defer_cross_file: bool,
) -> Vec<AppSelector> {
    let scoped_static_identifier_defaults = collect_scoped_static_identifier_defaults(program);
    let dynamic_identifier_values = match (visible_files, defer_cross_file) {
        (Some(visible), true) => {
            super::dynamic_values::collect_dynamic_identifier_values_with_file_from_visible_deferred(
                program, source, path, visible,
            )
        }
        (Some(visible), false) => {
            super::dynamic_values::collect_dynamic_identifier_values_with_file_from_visible(
                program, source, path, visible,
            )
        }
        (None, _) => super::dynamic_values::collect_dynamic_identifier_values_with_file(
            program, source, path,
        ),
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
        visible_files,
        selectors: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.selectors
}

include!("extract_app_visitor.rs");

#[cfg(test)]
mod tests;
