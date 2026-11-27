use super::*;

pub(in super::super::super) struct ReplayIndex {
    groups: FxHashMap<usize, FxHashMap<String, group::Group>>,
}
impl ReplayIndex {
    pub(in super::super::super) fn new(collector: &ImportCollector) -> Self {
        let mut groups: FxHashMap<usize, FxHashMap<String, group::Group>> = fx_map();
        for binding in &collector.value_flow.bindings {
            if let ValueExpression::Array(fields) = &binding.value {
                groups
                    .entry(binding.scope)
                    .or_default()
                    .entry(binding.name.clone())
                    .or_default()
                    .events
                    .insert(binding.declared_at, (std::sync::Arc::clone(fields), false));
            }
        }
        // Same-offset escaped facts take precedence, matching the original map.
        for (scope, name, at, fields, guarded) in &collector.receiver_escapes.arrays {
            groups
                .entry(*scope)
                .or_default()
                .entry(name.clone())
                .or_default()
                .events
                .insert(*at, (std::sync::Arc::clone(fields), *guarded));
        }
        for (scope, names) in &mut groups {
            for (name, group) in names {
                group.pushes = collector
                    .receiver_escapes
                    .pushes
                    .get(scope)
                    .and_then(|names| names.get(name))
                    .cloned()
                    .unwrap_or_default();
            }
        }
        Self { groups }
    }
    pub(in super::super::super) fn query(
        &mut self,
        collector: &ImportCollector,
        value: &ValueExpression,
        before: u32,
        recipient: Recipient,
    ) -> Option<Projection> {
        if let ValueExpression::Named { scope, name, .. } = value {
            self.groups
                .get_mut(&scope.unwrap_or(0))?
                .get_mut(name)?
                .query(&collector.value_flow.calls, before, recipient)
        } else {
            let baseline = entries(collector, value, before)?;
            let baseline_length = group::width(&baseline);
            Some(Projection {
                baseline,
                additional: Vec::new(),
                minimum: baseline_length,
                baseline_length,
                has_alternatives: false,
            })
        }
    }
}
