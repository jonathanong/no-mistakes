use super::{config, options, UnknownCalls};

#[test]
fn validation_rejects_empty_and_invalid_selectors() {
    for yaml in [
        "targets: [{ global: setTimeout }]",
        "roots: [{ file: src/entry.mts }]\nunknownCalls: ignore",
        "roots: [{ file: src/entry.mts }]\nmaxDepth: 0\ntargets: [{ global: setTimeout }]",
        "roots: [{ vitest: false }]\ntargets: [{ global: setTimeout }]",
        "roots: [{ file: src/entry.mts }]\ntargets: [{ moduleExport: { module: '', export: setTimeout } }]",
        "roots: [{ file: src/entry.mts }]\ntargets: [{ function: { file: '', symbol: run } }]",
    ] {
        assert!(config::validate(&options(yaml)).is_err(), "{yaml}");
    }
    assert!(config::validate(&options(
        "roots: [{ file: src/entry.mts }]\nunknownCalls: finding"
    ))
    .is_ok());
    let defaults = options("roots: [{ file: src/entry.mts }]");
    assert_eq!(defaults.unknown_calls, UnknownCalls::Finding);
    assert!(config::validate(&defaults).is_ok());
}

#[test]
fn validation_rejects_empty_vitest_project_lists_and_call_target_names() {
    for yaml in [
        "roots: [{ vitest: [] }]\ntargets: [{ global: setTimeout }]",
        "roots: [{ file: src/entry.mts }]\ntargets: [{ global: '' }]",
        "roots: [{ file: src/entry.mts }]\ntargets: [{ exact: '' }]",
        "roots: [{ file: src/entry.mts }]\ntargets: [{ terminal: '' }]",
    ] {
        let error = config::validate(&options(yaml)).expect_err("empty selector must be rejected");
        assert!(error.to_string().contains("must not be empty"), "{error:#}");
    }
}
