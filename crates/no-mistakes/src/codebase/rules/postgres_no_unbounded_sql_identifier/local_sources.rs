use super::*;
use crate::codebase::dependencies::extract::ValueType;

pub(super) fn local_env_alias(facts: &CheckFactMap, file: &Path, name: &str) -> bool {
    facts.get_ts_facts(file).is_some_and(|ts| {
        ts.value_flow
            .bindings
            .iter()
            .any(|binding| binding.name == name && expression_unbounded(&binding.value))
    })
}

/// A parameter of this exact scope that is not `number` typed. A name that
/// resolves to a plain local binding (or nowhere) also matches a same-named
/// parameter of another function, which keeps `const table = f(x)` reported
/// beside an unrelated `table` parameter; a loop binding does not.
pub(super) fn parameter(facts: &CheckFactMap, file: &Path, scope: usize, name: &str) -> bool {
    facts.get_ts_facts(file).is_some_and(|ts| {
        let flow = &ts.value_flow;
        let local = scope == 0
            || flow
                .bindings
                .iter()
                .any(|binding| binding.name == name && binding.scope == scope);
        flow.parameters.iter().any(|parameter| {
            parameter.name == name
                && if parameter.scope == scope {
                    !matches!(parameter.type_reference, Some(ValueType::Number))
                } else {
                    local
                }
        })
    })
}

pub(super) fn query_result_binding(facts: &CheckFactMap, file: &Path, name: &str) -> bool {
    facts.get_ts_facts(file).is_some_and(|ts| {
        ts.value_flow.bindings.iter().any(|binding| {
            binding.name == name
                && matches!(
                    binding.value,
                    ValueExpression::Call(_, _) | ValueExpression::Awaited(_)
                )
        })
    })
}

pub(super) fn export_is_unbounded(facts: &CheckFactMap, producer: &Path, exported: &str) -> bool {
    let Some(ts) = facts.get_ts_facts(producer) else {
        return false;
    };
    ts.value_flow.bindings.iter().any(|binding| {
        binding.owner.is_none() && binding.name == exported && expression_unbounded(&binding.value)
    })
}

pub(super) fn shadowed_process(facts: &CheckFactMap, file: &Path) -> bool {
    facts.get_ts_facts(file).is_some_and(|ts| {
        ts.imported_bindings.iter().any(|binding| {
            !binding.is_type_only
                && binding.local == "process"
                && !is_process_specifier(&binding.specifier)
        })
    })
}

fn expression_unbounded(expr: &ValueExpression) -> bool {
    let mut found = false;
    walk_expression(expr, &mut |value| {
        found |= is_process_env(value);
    });
    found
}

pub(super) fn imported_unbounded(
    root: &Path,
    facts: &CheckFactMap,
    call: &SqlCall<'_>,
    producer: std::path::PathBuf,
    exported: &str,
    name: &str,
) -> Option<RuleFinding> {
    if !export_is_unbounded(facts, &producer, exported) {
        return None;
    }
    let producer = relative_slash_path(root, &producer);
    Some(message(root, call, &producer, Some(name.to_string())))
}
