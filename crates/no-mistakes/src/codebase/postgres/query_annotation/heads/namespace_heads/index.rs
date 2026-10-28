use super::super::{syntax::Syntax, Span};
use crate::{
    codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts},
    fx::{fx_map, FxHashMap, FxHashSet},
};

pub(super) struct NamespaceIndexes<'a> {
    imports: FxHashMap<usize, (usize, String)>,
    locals: FxHashMap<Span, (usize, &'a ModuleBinding)>,
    exported_locals: FxHashSet<String>,
    pub(super) helper_calls: FxHashMap<Span, Vec<Span>>,
}

impl<'a> NamespaceIndexes<'a> {
    pub(super) fn build(
        syntax: &Syntax,
        module: &'a TypeScriptModuleFacts,
        executors: &FxHashSet<Span>,
    ) -> Self {
        let roots = module
            .scopes
            .iter()
            .filter(|scope| scope.parent_id.is_none())
            .map(|scope| scope.id)
            .collect::<FxHashSet<_>>();
        let mut indexes = Self {
            imports: fx_map(),
            locals: fx_map(),
            exported_locals: FxHashSet::default(),
            helper_calls: fx_map(),
        };
        let mut import_declarations_scanned = 0;
        let mut import_bindings_scanned = 0;
        for import in &module.imports {
            import_declarations_scanned += 1;
            if import.type_only {
                continue;
            }
            for binding in &import.bindings {
                import_bindings_scanned += 1;
                if !binding.type_only && binding.kind == "namespace" && binding.imported == "*" {
                    let entry = indexes
                        .imports
                        .entry(binding.binding_id)
                        .or_insert((0, import.specifier.clone()));
                    entry.0 += 1;
                }
            }
        }
        let mut module_bindings_scanned = 0;
        for binding in &module.bindings {
            module_bindings_scanned += 1;
            if !binding.imported && binding.runtime && roots.contains(&binding.scope_id) {
                let entry = indexes
                    .locals
                    .entry((binding.span.start, binding.span.end))
                    .or_insert((0, binding));
                entry.0 += 1;
            }
        }
        let mut exports_scanned = 0;
        for export in &module.exports {
            exports_scanned += 1;
            if !export.type_only && export.specifier.is_none() {
                indexes.exported_locals.insert(export.local.clone());
            }
        }
        let mut helper_arguments_scanned = 0;
        for (&call, &reference) in &syntax.helper_arguments {
            helper_arguments_scanned += 1;
            if executors.contains(&call) && !syntax.optional_calls.contains(&call) {
                indexes
                    .helper_calls
                    .entry(reference)
                    .or_default()
                    .push(call);
            }
        }
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.namespace_index_builds", 1);
            observer.increment(
                "postgres.annotation.namespace_import_declarations_scanned",
                import_declarations_scanned as u64,
            );
            observer.increment(
                "postgres.annotation.namespace_import_bindings_scanned",
                import_bindings_scanned as u64,
            );
            observer.increment(
                "postgres.annotation.namespace_module_bindings_scanned",
                module_bindings_scanned as u64,
            );
            observer.increment(
                "postgres.annotation.namespace_exports_scanned",
                exports_scanned as u64,
            );
            observer.increment(
                "postgres.annotation.namespace_helper_arguments_scanned",
                helper_arguments_scanned as u64,
            );
        }
        indexes
    }

    pub(super) fn namespace_import(&self, id: usize) -> Option<&str> {
        let (count, specifier) = self.imports.get(&id)?;
        (*count == 1).then_some(specifier)
    }

    pub(super) fn top_level_local(&self, span: Span) -> Option<&'a ModuleBinding> {
        let (count, binding) = self.locals.get(&span)?;
        (*count == 1).then_some(*binding)
    }

    pub(super) fn local_is_exported(&self, name: &str) -> bool {
        self.exported_locals.contains(name)
    }
}
