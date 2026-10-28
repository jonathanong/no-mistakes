//! One ambiguity-preserving owner and import projection for callback calls.

use super::Span;
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};

#[derive(Clone, Copy)]
struct Import<'a> {
    specifier: &'a str,
    imported: &'a str,
    local: &'a str,
}

pub(super) struct Imports<'a> {
    owners: FxHashMap<Span, Option<usize>>,
    bindings: FxHashMap<usize, Option<Import<'a>>>,
    local: FxHashMap<Span, Option<&'a ModuleBinding>>,
}

impl<'a> Imports<'a> {
    pub(super) fn new(module: &'a TypeScriptModuleFacts) -> Self {
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.callback_import_indexes", 1);
        }
        let mut owners = FxHashMap::default();
        let mut visits = 0;
        for binding in module
            .bindings
            .iter()
            .filter(|binding| binding.imported && binding.runtime)
        {
            let mut seen = FxHashSet::default();
            for reference in binding
                .references
                .iter()
                .filter(|reference| reference.runtime)
            {
                visits += 1;
                let span = (reference.span.start, reference.span.end);
                if seen.insert(span) {
                    owners
                        .entry(span)
                        .and_modify(|owner| *owner = None)
                        .or_insert(Some(binding.id));
                }
            }
        }
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment(
                "postgres.annotation.callback_import_reference_visits",
                visits,
            );
        }
        let mut bindings = FxHashMap::default();
        for import in module.imports.iter().filter(|import| !import.type_only) {
            for binding in import
                .bindings
                .iter()
                .filter(|binding| !binding.type_only && binding.imported != "*")
            {
                let value = Import {
                    specifier: import.specifier.as_str(),
                    imported: binding.imported.as_str(),
                    local: binding.local.as_str(),
                };
                bindings
                    .entry(binding.binding_id)
                    .and_modify(|known| *known = None)
                    .or_insert(Some(value));
            }
        }
        let roots = module
            .scopes
            .iter()
            .filter(|scope| scope.parent_id.is_none())
            .map(|scope| scope.id)
            .collect::<FxHashSet<_>>();
        let mut local = FxHashMap::default();
        let mut local_visits = 0;
        for binding in module.bindings.iter().filter(|binding| {
            binding.runtime && !binding.imported && roots.contains(&binding.scope_id)
        }) {
            for reference in binding.references.iter().filter(|usage| usage.runtime) {
                local_visits += 1;
                local
                    .entry((reference.span.start, reference.span.end))
                    .and_modify(|known| *known = None)
                    .or_insert(Some(binding));
            }
        }
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment(
                "postgres.annotation.callback_local_reference_visits",
                local_visits,
            );
        }
        Self {
            owners,
            bindings,
            local,
        }
    }

    fn binding(&self, span: Span) -> Option<Import<'a>> {
        let id = self.owners.get(&span).copied().flatten()?;
        self.bindings.get(&id).copied().flatten()
    }

    pub(super) fn imported(&self, span: Span) -> Option<(&'a str, &'a str)> {
        let import = self.binding(span)?;
        Some((import.specifier, import.imported))
    }

    pub(super) fn local_declaration(&self, span: Span) -> Option<Span> {
        self.local_owner(span)
            .map(|owner| (owner.span.start, owner.span.end))
    }

    pub(super) fn local_owner(&self, span: Span) -> Option<&'a ModuleBinding> {
        self.local.get(&span).copied().flatten()
    }

    pub(super) fn trusted_tag(&self, span: Span) -> bool {
        let Some(import) = self.binding(span) else {
            return false;
        };
        import.specifier == "sql-template-strings"
            && (import.imported == "default"
                || import.imported == "sql" && import.local.eq_ignore_ascii_case("sql"))
    }
}
