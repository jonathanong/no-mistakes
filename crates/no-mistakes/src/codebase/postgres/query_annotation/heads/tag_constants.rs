use super::{
    syntax::{Head, Syntax},
    Span,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashSet;

pub(super) fn safe(
    syntax: &Syntax,
    module: &TypeScriptModuleFacts,
    executor_calls: impl Iterator<Item = Span>,
) -> FxHashSet<Span> {
    let executors = executor_calls.collect::<FxHashSet<_>>();
    let allowed = syntax
        .calls
        .iter()
        .filter_map(|call| match call.head {
            Head::Reference(span) if executors.contains(&call.span) => Some(span),
            _ => None,
        })
        .collect::<FxHashSet<_>>();
    let roots = module
        .scopes
        .iter()
        .filter(|scope| scope.parent_id.is_none())
        .map(|scope| scope.id)
        .collect::<FxHashSet<_>>();
    let exports = module
        .exports
        .iter()
        .filter(|export| export.specifier.is_none())
        .map(|export| export.local.as_str())
        .collect::<FxHashSet<_>>();
    module
        .bindings
        .iter()
        .filter_map(|binding| {
            let declaration = (binding.span.start, binding.span.end);
            if !binding.runtime
                || binding.imported
                || !matches!(syntax.constants.get(&declaration), Some(Head::Tag(_)))
                || (roots.contains(&binding.scope_id) && exports.contains(binding.name.as_str()))
            {
                return None;
            }
            // A const binding does not make a tag's returned builder immutable.
            // Every runtime use must hand it directly to a prepared executor.
            binding
                .references
                .iter()
                .filter(|usage| usage.runtime)
                .all(|usage| allowed.contains(&(usage.span.start, usage.span.end)))
                .then_some(declaration)
        })
        .collect()
}
