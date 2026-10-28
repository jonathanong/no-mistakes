//! Exact local call census for one finite helper-name edge.
use super::{Argument, CallSite, Root};
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings, syntax::Syntax, Span,
};
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};

pub(super) fn helper_domain(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: &TypeScriptModuleFacts,
    root: &Root,
) -> Option<Vec<String>> {
    let function = syntax
        .finite_functions
        .iter()
        .find(|f| f.body == root.function)?;
    let helper = declared(module, function.declaration)?;
    let parameter = declared(module, function.parameter)?;
    let name_owner = bindings.owner(root.reference)?;
    if name_owner.id != parameter.id
        || !stable(syntax, bindings, module, helper)
        || !stable(syntax, bindings, module, parameter)
        || syntax.append_tags.get(&root.declaration) != Some(&root.tag)
        || !bindings.trusted_root(root.declaration)
    {
        return None;
    }
    let calls = direct_calls(syntax, bindings, helper);
    let [call] = calls.as_slice() else {
        return None;
    };
    if !only_references(helper, &[call.callee]) || !call.valid || call.count == 0 {
        return None;
    }
    let Argument::Reference(argument) = &call.first else {
        return None;
    };
    let select = syntax
        .finite_selects
        .iter()
        .find(|s| Some(s.body) == call.context)?;
    let select_parameter = declared(module, select.parameter)?;
    if bindings
        .owner(*argument)
        .is_none_or(|owner| owner.id != select_parameter.id)
        || !stable(syntax, bindings, module, select_parameter)
    {
        return None;
    }
    let select_binding = declared(module, select.declaration)?;
    if !stable(syntax, bindings, module, select_binding) {
        return None;
    }
    let calls = direct_calls(syntax, bindings, select_binding);
    if calls.is_empty()
        || calls.len() > 16
        || !only_references(
            select_binding,
            &calls.iter().map(|c| c.callee).collect::<Vec<_>>(),
        )
    {
        return None;
    }
    let mut values = Vec::new();
    for call in calls {
        if !call.valid || call.count != 1 || call.context != Some(select.enclosing) {
            return None;
        }
        let Argument::Literal(value) = &call.first else {
            return None;
        };
        if value.trim().is_empty() {
            return None;
        }
        if !values.contains(value) {
            values.push(value.clone());
        }
    }
    Some(values)
}

fn declared(module: &TypeScriptModuleFacts, span: Span) -> Option<&ModuleBinding> {
    let mut matching = module.bindings.iter().filter(|binding| {
        (binding.span.start, binding.span.end) == span && binding.runtime && !binding.imported
    });
    let binding = matching.next()?;
    matching.next().is_none().then_some(binding)
}
fn direct_calls<'a>(
    syntax: &'a Syntax,
    bindings: &Bindings<'_>,
    target: &ModuleBinding,
) -> Vec<&'a CallSite> {
    syntax
        .finite_calls
        .iter()
        .filter(|call| {
            bindings
                .owner(call.callee)
                .is_some_and(|owner| owner.id == target.id)
        })
        .collect()
}
fn only_references(binding: &ModuleBinding, expected: &[Span]) -> bool {
    binding
        .references
        .iter()
        .filter(|usage| usage.runtime)
        .count()
        == expected.len()
        && binding
            .references
            .iter()
            .filter(|usage| usage.runtime)
            .all(|usage| expected.contains(&(usage.span.start, usage.span.end)))
}
pub(super) fn stable(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: &TypeScriptModuleFacts,
    binding: &ModuleBinding,
) -> bool {
    let declaration = (binding.span.start, binding.span.end);
    if !bindings.declaration_safe(declaration)
        || module.exports.iter().any(|export| {
            export.specifier.is_none() && !export.type_only && export.local == binding.name
        })
    {
        return false;
    }
    binding
        .references
        .iter()
        .filter(|usage| usage.runtime)
        .all(|usage| {
            let span = (usage.span.start, usage.span.end);
            bindings
                .owner(span)
                .is_some_and(|owner| owner.id == binding.id)
                && !syntax.assignments.unsupported_writes.contains(&span)
                && !syntax.assignments.writes.contains_key(&span)
        })
}
