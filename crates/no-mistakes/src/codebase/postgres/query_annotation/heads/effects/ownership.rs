//! Expand the shared owner index once for complete effect-reference censuses.
use super::Span;
use crate::codebase::postgres::query_annotation::heads::syntax::Syntax;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashSet;

pub(in super::super) fn extend(
    syntax: &Syntax,
    module: &TypeScriptModuleFacts,
    wanted: &mut FxHashSet<Span>,
) {
    if syntax.effect_functions.is_empty() {
        return;
    }
    let declarations = syntax
        .effect_functions
        .iter()
        .flat_map(|(declaration, summaries)| {
            std::iter::once(*declaration).chain(summaries.iter().flatten().map(|s| s.parameter))
        })
        .collect::<FxHashSet<_>>();
    let callees = syntax
        .effect_calls
        .iter()
        .map(|call| call.callee)
        .collect::<FxHashSet<_>>();
    for binding in &module.bindings {
        if declarations.contains(&(binding.span.start, binding.span.end))
            || (binding.imported
                && binding
                    .references
                    .iter()
                    .any(|usage| callees.contains(&(usage.span.start, usage.span.end))))
        {
            wanted.extend(
                binding
                    .references
                    .iter()
                    .filter(|usage| usage.runtime)
                    .map(|usage| (usage.span.start, usage.span.end)),
            );
        }
    }
}
