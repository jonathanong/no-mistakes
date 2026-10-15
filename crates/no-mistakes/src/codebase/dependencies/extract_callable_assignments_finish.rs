fn finish_callable_assignment_facts(collector: &mut ImportCollector) {
    if !collector.value_flow.callable_assignments.is_empty() {
        let candidates = collector
            .value_flow
            .callable_assignments
            .iter()
            .filter(|write| write.callable.is_some())
            .map(|write| (write.scope, write.name.clone()))
            .collect::<FxHashSet<_>>();
        collector
            .value_flow
            .callable_assignments
            .retain(|write| candidates.contains(&(write.scope, write.name.clone())));
        collector.value_flow.callable_assignment_uses = collector
            .callable_assignment_uses
            .iter()
            .filter(|(scope, name)| candidates.contains(&(*scope, name.clone())))
            .cloned()
            .collect();
    }
}
