use super::*;

pub(super) fn local_env_alias(facts: &CheckFactMap, file: &Path, name: &str) -> bool {
    facts.get_ts_facts(file).is_some_and(|ts| {
        ts.value_flow
            .bindings
            .iter()
            .any(|binding| binding.name == name && expression_unbounded(&binding.value))
    })
}

pub(super) fn parameter(facts: &CheckFactMap, file: &Path, name: &str) -> bool {
    facts.get_ts_facts(file).is_some_and(|ts| {
        ts.value_flow
            .parameters
            .iter()
            .any(|parameter| parameter.name == name)
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

fn expression_unbounded(expr: &ValueExpression) -> bool {
    let mut found = false;
    walk_expression(expr, &mut |value| {
        found |= is_process_env(value);
    });
    found
}
