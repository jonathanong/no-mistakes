use super::*;

impl ImportCollector {
    pub(in super::super) fn invalidate_value_write(
        &mut self,
        scope: usize,
        name: &str,
        offset: u32,
    ) {
        let (base, member) = name
            .split_once('.')
            .map_or((name, false), |(base, _)| (base, true));
        let mut affected = fx_set();
        affected.insert((scope, base.to_string()));
        if member {
            let mut links: FxHashMap<(usize, String), Vec<(usize, String)>> = fx_map();
            for binding in &self.value_flow.bindings {
                if binding
                    .invalidated_at
                    .is_some_and(|cutoff| cutoff <= offset)
                {
                    continue;
                }
                if let Some(target) = referenced_value_root(&binding.value) {
                    let local = (binding.scope, binding.name.clone());
                    links.entry(local.clone()).or_default().push(target.clone());
                    links.entry(target).or_default().push(local);
                }
            }
            let mut pending = vec![(scope, base.to_string())];
            while let Some(key) = pending.pop() {
                for target in links.get(&key).into_iter().flatten() {
                    if affected.insert(target.clone()) {
                        pending.push(target.clone());
                    }
                }
            }
        }
        for (scope, name) in &affected {
            for index in self
                .value_binding_indexes
                .get(scope)
                .and_then(|names| names.get(name.as_str()))
                .into_iter()
                .flatten()
            {
                self.value_flow.bindings[*index]
                    .invalidated_at
                    .get_or_insert(offset);
            }
            for index in self
                .value_parameter_indexes
                .get(scope)
                .and_then(|names| names.get(name.as_str()))
                .into_iter()
                .flatten()
            {
                self.value_flow.parameters[*index]
                    .invalidated_at
                    .get_or_insert(offset);
            }
        }
        let mut invalidated = Vec::new();
        for binding in &mut self.callable_aliases {
            let alias = &mut binding.alias;
            let head = alias
                .local
                .split_once('.')
                .map_or(alias.local.as_str(), |(head, _)| head);
            if member && affected.contains(&(alias.binding_scope, head.to_string())) {
                alias.invalidated_at.get_or_insert(offset);
                invalidated.push((alias.binding_scope, alias.local.clone()));
            }
        }
        for (scope, local) in invalidated {
            self.insert_reassigned_callable_at(scope, local);
        }
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
