use super::*;

impl ImportCollector {
    pub(super) fn prepare_value_references(&mut self) {
        self.prepare_parameter_origins();
        if self.value_reference_links.is_some() {
            return;
        }
        let mut links = fx_map();
        for binding in &self.value_flow.bindings {
            insert_reference(
                &mut links,
                &mut self.prior_member_mutations,
                binding.scope,
                &binding.name,
                &binding.value,
            );
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.reference_link_constructions += 1;
            }
        }
        self.value_reference_links = Some(links);
    }
    pub(super) fn record_value_reference(
        &mut self,
        scope: usize,
        name: &str,
        value: &ValueExpression,
    ) {
        if let Some(links) = &mut self.value_reference_links {
            insert_reference(links, &mut self.prior_member_mutations, scope, name, value);
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.reference_link_constructions += 1;
            }
        }
    }
}
fn insert_reference(
    links: &mut ReferenceLinks,
    histories: &mut PriorMemberMutations,
    scope: usize,
    name: &str,
    value: &ValueExpression,
) {
    if nested_reference_container(value) {
        histories.require_reference_proof(scope, name);
    }
    if let Some((target_scope, target_name)) = referenced_value_root(value) {
        histories.link((scope, name), (target_scope, &target_name));
        links
            .entry(scope)
            .or_default()
            .entry(name.to_string())
            .or_default()
            .push((target_scope, target_name.clone()));
        links
            .entry(target_scope)
            .or_default()
            .entry(target_name)
            .or_default()
            .push((scope, name.to_string()));
    }
}

fn referenced_value_root(value: &ValueExpression) -> Option<(usize, String)> {
    match value {
        ValueExpression::Named { name, scope, .. } if !name.contains('.') => Some((
            scope.unwrap_or(0),
            name.split_once('.')
                .map_or(name.as_str(), |(head, _)| head)
                .to_string(),
        )),

        _ => None,
    }
}

fn nested_reference_container(value: &ValueExpression) -> bool {
    match value {
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
            fields.values().any(has_reference)
        }
        ValueExpression::Default(value, fallback) => {
            nested_reference_container(value) || nested_reference_container(fallback)
        }
        ValueExpression::Member(_, _) => true,
        ValueExpression::Named { name, .. } => name.contains('.'),
        ValueExpression::Awaited(value) => nested_reference_container(value),
        _ => false,
    }
}
fn has_reference(value: &ValueExpression) -> bool {
    matches!(value, ValueExpression::Named { .. }) || nested_reference_container(value)
}
