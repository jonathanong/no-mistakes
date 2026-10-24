use super::*;

pub(super) fn run_enabled_rules(inputs: &RuleRunInputs<'_>) {
    macro_rules! run_rules { ($($id:expr => $call:path),* $(,)?) => { rayon::scope(|scope| { $( if rule_enabled(inputs.config, $id) { scope.spawn(|_| { let result = trace_rule(inputs.sources, $id, || run_rule::run_rule_with_sources(run_rule::RunRuleRequest { rule_id: $id, fallback: $call, root: inputs.root, config: inputs.config, files: inputs.candidates.candidates($id), snapshot: inputs.snapshot, next_config_resolver: inputs.next_config_resolver, sources: inputs.sources, facts: inputs.facts, defer_suppression: inputs.defer_suppression })); inputs.acc.lock().expect("mutex poisoned").push(($id, result)); }); } )*; special::spawn(scope, inputs); }); }; }
    crate::filesystem_rules!(run_rules);
}

pub(in crate::codebase::rules::filesystem_dispatch) fn trace_rule<T>(
    sources: &crate::codebase::ts_source::SourceStore,
    rule_id: &str,
    run: impl FnOnce() -> T,
) -> T {
    // Rayon rule tasks must install the source owner's observer before nested
    // application workers capture it for their own work counters and spans.
    crate::diagnostics::with_observer(sources.observer().cloned(), || {
        timing::trace_rule(
            sources.observer().map(|observer| observer.as_ref()),
            || format!("rules.{rule_id}"),
            run,
        )
    })
}
