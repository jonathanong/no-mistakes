//! `export const name = ["parent > child", ...]` modules named by `includeSources`.

use super::include_config::{includes_in_array, NestedInclude};
use super::{IncludeSource, RuleFinding, RULE_ID};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use oxc_ast::ast::{BindingPattern, Declaration, Expression, Statement, VariableDeclarationKind};
use oxc_span::GetSpan;
use std::path::Path;

pub(super) enum SourceList {
    Includes(Vec<NestedInclude>),
    Unanalyzable { line: usize, reason: String },
}

pub(super) fn load(
    root: &Path,
    sources: &SourceStore,
    include_sources: &[IncludeSource],
    includes: &mut Vec<(String, NestedInclude)>,
    findings: &mut Vec<RuleFinding>,
    configs_known: &mut bool,
) {
    for spec in include_sources {
        if spec.module.is_empty() || spec.export.is_empty() {
            *configs_known = false;
            findings.push(gap(
                "includeSources",
                1,
                "includeSources entries need a module and an export name",
            ));
            continue;
        }
        let path = crate::codebase::ts_resolver::normalize_path(&root.join(&spec.module));
        let file = relative_slash_path(root, &path);
        match sources.read_source_path(&path) {
            Ok(source) => match parse_include_source(&path, &source, &spec.export) {
                SourceList::Includes(entries) => {
                    includes.extend(entries.into_iter().map(|entry| (file.clone(), entry)));
                }
                SourceList::Unanalyzable { line, reason } => {
                    *configs_known = false;
                    findings.push(gap(&file, line, &reason));
                }
            },
            Err(error) => {
                *configs_known = false;
                findings.push(super::super::source_access::read_failure(
                    root, &path, RULE_ID, &error,
                ));
            }
        }
    }
}

fn gap(file: &str, line: usize, reason: &str) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!(
            "cannot verify {RULE_ID}: {reason}; export a static array of string literals from includeSources so this check can read nested includes"
        ),
        import: None,
        target: Some("unanalyzable".to_string()),
    }
}

pub(super) fn parse_include_source(path: &Path, source: &str, export_name: &str) -> SourceList {
    match crate::ast::with_program(path, source, |program, source| {
        extract(program, source, export_name)
    }) {
        Ok(parsed) => parsed,
        Err(_) => SourceList::Unanalyzable {
            line: 1,
            reason: format!("cannot parse include source export {export_name}"),
        },
    }
}

fn extract(program: &oxc_ast::ast::Program<'_>, source: &str, export_name: &str) -> SourceList {
    for statement in &program.body {
        let Statement::ExportDeclaration(export) = statement else {
            continue;
        };
        let Declaration::VariableDeclaration(declaration) = &export.declaration else {
            continue;
        };
        if declaration.kind != VariableDeclarationKind::Const {
            continue;
        }
        for declarator in &declaration.declarations {
            let Some(name) = binding_name(&declarator.id) else {
                continue;
            };
            if name != export_name {
                continue;
            }
            let Some(init) = declarator.init.as_ref() else {
                return missing(export_name);
            };
            let Expression::ArrayExpression(array) = init else {
                return SourceList::Unanalyzable {
                    line: line_of(source, init.span().start),
                    reason: format!("export {export_name} is not a static array of strings"),
                };
            };
            return match includes_in_array(source, array.as_ref()) {
                Ok(entries) => SourceList::Includes(entries),
                Err((line, reason)) => SourceList::Unanalyzable { line, reason },
            };
        }
    }
    missing(export_name)
}

fn missing(export_name: &str) -> SourceList {
    SourceList::Unanalyzable {
        line: 1,
        reason: format!("include source does not export const {export_name}"),
    }
}

fn binding_name<'a>(binding: &'a BindingPattern<'a>) -> Option<&'a str> {
    match binding {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn line_of(source: &str, start: u32) -> usize {
    crate::codebase::ts_source::byte_offset_to_line(source, start as usize) as usize
}
