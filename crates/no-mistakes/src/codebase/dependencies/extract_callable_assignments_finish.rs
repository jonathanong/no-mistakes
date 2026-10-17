fn finish_callable_assignment_facts(collector: &mut ImportCollector) {
    let Some(mut state) = collector.callable_assignment_state.take() else {
        return;
    };
    let mut candidates: FxHashMap<usize, FxHashSet<String>> = fx_map();
    for write in state
        .assignments
        .iter()
        .filter(|write| write.callable.is_some())
    {
        candidates
            .entry(write.scope)
            .or_default()
            .insert(write.name.clone());
    }
    state.assignments.retain(|write| {
        candidates
            .get(&write.scope)
            .is_some_and(|names| names.contains(&write.name))
    });
    if state.assignments.is_empty() {
        return;
    }
    let capsule = collector.value_flow.replacements.get_or_insert_with(|| {
        std::sync::Arc::new(ValueReplacementFacts {
            callable_only: true,
            ..Default::default()
        })
    });
    let facts = std::sync::Arc::get_mut(capsule).expect("owned callable facts");
    facts.callable_assignments = state.assignments;
    for (scope, name) in state.uses {
        if candidates
            .get(&scope)
            .is_some_and(|names| names.contains(&name))
        {
            facts
                .callable_assignment_uses
                .entry(scope)
                .or_default()
                .insert(name);
        }
    }
}
