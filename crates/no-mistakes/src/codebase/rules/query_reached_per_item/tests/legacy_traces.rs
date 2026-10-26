// Expand the oracle hook only in test builds; production has no branch or state.
macro_rules! legacy_reachable {
    ($graph:expr, $roots:expr, $sinks:expr, $excluded:expr) => {
        if super::tests::sink_paths::USE_LEGACY.with(|mode| mode.get()) {
            return super::tests::sink_paths::legacy_reachable($graph, $roots, $sinks, $excluded);
        }
    };
}
