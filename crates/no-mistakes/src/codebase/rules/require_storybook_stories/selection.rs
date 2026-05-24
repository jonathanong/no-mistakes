use super::types::{
    component_key, is_react_source_file, source_has_required_prop, Component, GlobMatcher, Options,
};
use super::RULE_ID;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{
    has_disable_comment, has_disable_file_comment, relative_slash_path,
};
use crate::codebase::ts_symbols::ExportKind;
use std::path::Path;

pub(super) fn selected_components(
    root: &Path,
    project_root: &Path,
    shared: &CheckFactMap,
    opts: &Options,
    include: &GlobMatcher,
    exclude: &GlobMatcher,
    test_filter: &crate::codebase::test_filter::TestFileFilter,
) -> Vec<Component> {
    let mut components = Vec::new();
    for (path, facts) in &shared.ts {
        if !path.starts_with(project_root) || !is_indexable(path) || !is_react_source_file(path) {
            continue;
        }
        if test_filter.is_match(root, path) {
            continue;
        }
        if opts.ignore_index_and_private_files && is_index_or_private_file(path) {
            continue;
        }
        let project_file = relative_slash_path(project_root, path);
        if exclude.is_match(&project_file) {
            continue;
        }
        let explicit = include.is_match(&project_file);
        if facts.parse_error.is_some() {
            continue;
        }
        let Some(react) = facts.react.as_ref() else {
            continue;
        };
        if should_skip_file(facts, opts, explicit) {
            continue;
        }
        for component in &react.components {
            let include_by_kind = if component.name == "default" {
                opts.include_all_react_default_exports
            } else {
                opts.include_all_react_named_exports
            };
            if !explicit && !include_by_kind {
                continue;
            }
            let line = export_line(facts, &component.name).unwrap_or(1) as usize;
            if component_disabled(shared, path, line) {
                continue;
            }
            components.push(Component {
                key: component_key(&project_file, &component.name),
                file: normalize_path(path),
                repo_file: relative_slash_path(root, path),
                project_file: project_file.clone(),
                export_name: component.name.clone(),
                line,
                explicit,
            });
        }
    }
    components.sort_by_key(|c| (c.project_file.clone(), c.export_name.clone()));
    components.dedup_by_key(|c| c.key.clone());
    components
}

fn should_skip_file(facts: &CheckFileFacts, opts: &Options, explicit: bool) -> bool {
    let Some(source) = facts.source.as_deref() else {
        return false;
    };
    has_disable_file_comment(source, RULE_ID)
        || (!explicit && !opts.required_props.is_empty() && !source_has_required_prop(source, opts))
}

fn is_index_or_private_file(path: &Path) -> bool {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| stem == "index" || stem.starts_with('_'))
}

fn export_line(facts: &CheckFileFacts, export_name: &str) -> Option<u32> {
    let symbols = facts.symbols.as_ref()?;
    symbols.exports.iter().find_map(|export| {
        if export_name == "default" {
            matches!(export.kind, ExportKind::Default).then_some(export.line)
        } else {
            (export.name == export_name && !export.is_type_only).then_some(export.line)
        }
    })
}

pub(super) fn component_disabled(shared: &CheckFactMap, file: &Path, line: usize) -> bool {
    shared
        .ts
        .get(file)
        .and_then(|facts| facts.source.as_deref())
        .is_some_and(|source| has_disable_comment(source, line as u32, RULE_ID))
}

pub(super) fn file_disabled(shared: &CheckFactMap, file: &Path) -> bool {
    shared
        .ts
        .get(file)
        .and_then(|facts| facts.source.as_deref())
        .is_some_and(|source| has_disable_file_comment(source, RULE_ID))
}
