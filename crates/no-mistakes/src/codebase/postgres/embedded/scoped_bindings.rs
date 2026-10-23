//! Lexically scoped executor bindings: locals bound to a configured factory
//! call, and function parameters typed with a configured executor type.
//!
//! Unlike the file-wide import names in `bindings.rs`, these only apply
//! inside the declaring block (variables) or function (parameters), stored as
//! source spans so a same-named identifier elsewhere is never matched.

use super::relative::{PendingRelativeSpan, RelativeScopedCandidate};
use super::EmbeddedSqlOptions;
use candidates::classify;
use collector::ScopeCollector;
use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Program, Statement};
use oxc_ast_visit::Visit;
use oxc_span::Span;
use std::collections::HashMap;

mod candidates;
mod collector;
mod owners;

/// The module itself or any subpath of it (`@example/db/types`); a sibling
/// package sharing the prefix (`@example/dbx`) does not match.
pub(crate) fn from_configured_module(source: &str, specifier: &str) -> bool {
    candidates::from_configured_module(source, specifier)
}

pub(super) struct ScopedCollection {
    #[cfg(test)]
    pub(super) delayed_records: usize,
    pub(super) executors: ScopedExecutors,
    pub(super) candidates: Vec<RelativeScopedCandidate>,
    pub(super) spans: Vec<PendingRelativeSpan>,
}

impl ScopedCollection {
    /// Unused helper imports must not initialize a resolver projection.
    pub(super) fn prune_unused_diagnostic_candidates(
        &mut self,
        calls: &[super::UndeclaredExecutorType],
    ) {
        if calls.is_empty()
            && self
                .candidates
                .iter()
                .all(|candidate| !candidate.factory && !candidate.type_import)
        {
            self.candidates.clear();
        }
    }
}

pub(super) type UndeclaredReceiverTypes = HashMap<String, super::UndeclaredExecutorType>;

/// Binding name to the source spans in which it is an executor.
#[derive(Debug, Default)]
pub(super) struct ScopedExecutors {
    spans: HashMap<String, Vec<Span>>,
    pub(super) delayed: std::collections::HashSet<u32>,
    pub(super) undeclared: HashMap<u32, HashMap<String, UndeclaredReceiverTypes>>,
    pub(super) undeclared_this: Vec<(Span, UndeclaredReceiverTypes)>,
    pub(super) undeclared_this_exclusions: Vec<Span>,
    pub(super) undeclared_imports: HashMap<String, super::UndeclaredExecutorType>,
}

impl ScopedExecutors {
    pub(super) fn contains_declaration(&self, name: &str, start: u32) -> bool {
        self.spans
            .get(name)
            .is_some_and(|spans| spans.iter().any(|span| span.start == start))
    }

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

pub(super) fn collect(program: &Program<'_>, options: &EmbeddedSqlOptions) -> ScopedCollection {
    let mut imports = classify(program, options);
    let candidates = std::mem::take(&mut imports.candidates);
    // Negative-only preflight: every supported binding spells `let`. Comments
    // and unrelated identifiers may enable extra work, but never prove a receiver.
    let mut visitor = ScopeCollector::from_imports(imports, program.source_text.contains("let"));
    if visitor.needs_walk() {
        visitor.visit_program(program);
        visitor.finish_undeclared_annotations();
        visitor.finish_delayed_receivers();
    }
    ScopedCollection {
        #[cfg(test)]
        delayed_records: visitor.delayed_records(),
        executors: visitor.found,
        candidates,
        spans: visitor.spans,
    }
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
        if !candidates::from_configured_module(
            import.source.value.as_str(),
            &options.import_specifier,
        ) {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
                continue;
            };
            let imported = named.imported.name();
            let type_only = import.import_kind == ImportOrExportKind::Type
                || named.import_kind == ImportOrExportKind::Type;
            if !type_only
                && candidates::contains(&options.executor_factory_names, imported.as_str())
            {
                factories.push(imported.to_string());
            }
            if candidates::contains(&options.executor_type_names, imported.as_str()) {
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
