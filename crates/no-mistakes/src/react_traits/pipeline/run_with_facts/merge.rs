fn merge_component(agg: &mut AggregateResult, facts: &ComponentFacts) {
    agg.facts.has_state |= facts.has_state;
    agg.facts.has_props |= facts.has_props;
    agg.facts.passes_props |= facts.passes_props;
    agg.facts.uses_memo |= facts.uses_memo;
    agg.facts.uses_context_provider |= facts.uses_context_provider;
    agg.facts.uses_suspense |= facts.uses_suspense;
    agg.facts.has_fetch |= !facts.fetches.is_empty();
    agg.fetch_locations.extend(
        facts
            .fetches
            .iter()
            .map(|fetch| (fetch.file.clone(), fetch.line)),
    );
}

fn merge_aggregate(agg: &mut AggregateResult, child: &AggregateResult) {
    agg.facts.has_state |= child.facts.has_state;
    agg.facts.has_fetch |= child.facts.has_fetch;
    agg.facts.uses_suspense |= child.facts.uses_suspense;
    agg.facts.uses_context_provider |= child.facts.uses_context_provider;
    agg.facts.uses_memo |= child.facts.uses_memo;
    agg.facts.has_props |= child.facts.has_props;
    agg.facts.passes_props |= child.facts.passes_props;
    agg.fetch_locations
        .extend(child.fetch_locations.iter().cloned());
}
