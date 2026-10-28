//! Finite callback inputs supplied by the configured Vitest `it.each`/`test.each` API.
mod collector;
use crate::codebase::{
    dependencies::extract::CallableId,
    postgres::embedded::walk::resolve::functions::provenance::RecoveredSql,
    ts_source::module_facts::TypeScriptModuleFacts,
};
use crate::fx::{FxHashMap, FxHashSet};
use collector::Collector;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;

/// Map each inline callback to its finite rows, retaining the authored literal positions.
pub(crate) fn collect(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    source: &str,
) -> FxHashMap<CallableId, Vec<RecoveredSql>> {
    // A second import path can mutate the same runner's `.each` property.
    if module.imports.iter().any(|import| {
        import.specifier == "vitest"
            && !import.type_only
            && import
                .bindings
                .iter()
                .any(|binding| !binding.type_only && binding.kind != "named")
    }) {
        return FxHashMap::default();
    }
    let runners = vitest_runner_bindings(module);
    if runners.is_empty() {
        return FxHashMap::default();
    }
    let mut visitor = Collector {
        source,
        slots: binding_slots(module),
        runners,
        sites: FxHashMap::default(),
        runner_uses: FxHashSet::default(),
        unsafe_file: false,
    };
    visitor.visit_program(program);
    let all_uses_are_calls = module
        .bindings
        .iter()
        .filter(|binding| visitor.runners.contains(&binding.id))
        .flat_map(|binding| &binding.references)
        .filter(|reference| reference.runtime)
        .all(|reference| {
            visitor
                .runner_uses
                .contains(&(reference.span.start, reference.span.end))
        });
    if visitor.unsafe_file || !all_uses_are_calls {
        FxHashMap::default()
    } else {
        visitor.sites
    }
}

fn vitest_runner_bindings(module: &TypeScriptModuleFacts) -> FxHashSet<usize> {
    module
        .imports
        .iter()
        .filter(|import| import.specifier == "vitest" && !import.type_only)
        .flat_map(|import| &import.bindings)
        .filter(|binding| {
            !binding.type_only
                && binding.kind == "named"
                && matches!(binding.imported.as_str(), "it" | "test")
        })
        .map(|binding| binding.binding_id)
        .collect()
}

fn binding_slots(module: &TypeScriptModuleFacts) -> FxHashMap<(u32, u32), usize> {
    let mut slots = FxHashMap::default();
    for binding in &module.bindings {
        slots.insert((binding.span.start, binding.span.end), binding.id);
        for reference in &binding.references {
            if reference.runtime {
                slots.insert((reference.span.start, reference.span.end), binding.id);
            }
        }
    }
    slots
}

#[cfg(test)]
#[path = "iterator_tests.rs"]
mod tests;
