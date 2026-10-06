//! Lexically scoped executor bindings: locals bound to a configured factory
//! call, and function parameters typed with a configured executor type.
//!
//! Unlike the file-wide import names in `bindings.rs`, these only apply
//! inside the declaring block (variables) or function (parameters), stored as
//! source spans so a same-named identifier elsewhere is never matched.

use super::EmbeddedSqlOptions;
use collector::ScopeCollector;
use oxc_ast::ast::{
    ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind, Program, Statement,
};
use oxc_ast_visit::Visit;
use oxc_span::Span;
use std::collections::HashMap;

mod collector;

/// Binding name to the source spans in which it is an executor.
#[derive(Debug, Default)]
pub(super) struct ScopedExecutors {
    spans: HashMap<String, Vec<Span>>,
}

impl ScopedExecutors {
    pub(super) fn contains(&self, name: &str, position: u32) -> bool {
        self.spans.get(name).is_some_and(|spans| {
            spans
                .iter()
                .any(|span| span.start <= position && position < span.end)
        })
    }

    pub(super) fn add(&mut self, name: &str, span: Span) {
        self.spans.entry(name.to_string()).or_default().push(span);
    }
}

pub(super) fn scoped_executors(
    program: &Program<'_>,
    options: &EmbeddedSqlOptions,
) -> ScopedExecutors {
    let mut visitor = ScopeCollector::default();
    for statement in &program.body {
        if let Statement::ImportDeclaration(import) = statement {
            collect_imports(import, options, &mut visitor);
        }
    }
    if !(visitor.factories.is_empty() && visitor.types.is_empty()) {
        visitor.visit_program(program);
    }
    visitor.found
}

/// Configured scoped names this file imports from the configured module,
/// as `(factory names, type names)`, using the same matching as the collector.
pub(super) fn matched_names(
    program: &Program<'_>,
    options: &EmbeddedSqlOptions,
) -> (Vec<String>, Vec<String>) {
    let (mut factories, mut types) = (Vec::new(), Vec::new());
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if !from_configured_module(import.source.value.as_str(), &options.import_specifier) {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
                continue;
            };
            let imported = named.imported.name();
            let type_only = import.import_kind == ImportOrExportKind::Type
                || named.import_kind == ImportOrExportKind::Type;
            if !type_only && contains(&options.executor_factory_names, imported.as_str()) {
                factories.push(imported.to_string());
            }
            if contains(&options.executor_type_names, imported.as_str()) {
                types.push(imported.to_string());
            }
        }
    }
    (sorted_unique(factories), sorted_unique(types))
}

fn sorted_unique(mut names: Vec<String>) -> Vec<String> {
    names.sort();
    names.dedup();
    names
}

fn collect_imports(
    import: &ImportDeclaration<'_>,
    options: &EmbeddedSqlOptions,
    visitor: &mut ScopeCollector,
) {
    if !from_configured_module(import.source.value.as_str(), &options.import_specifier) {
        return;
    }
    let Some(specifiers) = &import.specifiers else {
        return;
    };
    for specifier in specifiers {
        let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
            continue;
        };
        let imported = named.imported.name();
        let local = named.local.name.to_string();
        let type_only = import.import_kind == ImportOrExportKind::Type
            || named.import_kind == ImportOrExportKind::Type;
        if !type_only && contains(&options.executor_factory_names, imported.as_str()) {
            visitor.factories.insert(local.clone());
        }
        if contains(&options.executor_type_names, imported.as_str()) {
            visitor.types.insert(local);
        }
    }
}

/// The module itself or any subpath of it (`@example/db/types`); a sibling
/// package sharing the prefix (`@example/dbx`) does not match.
pub(super) fn from_configured_module(source: &str, specifier: &str) -> bool {
    specifier.is_empty()
        || source == specifier
        || source
            .strip_prefix(specifier)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn contains(names: &[String], name: &str) -> bool {
    names.iter().any(|candidate| candidate == name)
}
