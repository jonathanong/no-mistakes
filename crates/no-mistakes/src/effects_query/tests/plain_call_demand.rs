use super::*;
use crate::codebase::dependencies::graph::GraphFiles;
use crate::codebase::ts_source::facts::{TsFactMap, TsFactPlan, TsFileFacts};

fn plain_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/query-reached-per-item/plain-alternatives-fixture")
}

fn foreign_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/effects-plain-foreign-write/fixture")
}

fn plain_selection() -> EffectsSelection {
    let root = plain_fixture();
    let config: crate::config::v2::NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    selection_from_config(&config, "plain", &[]).unwrap()
}

#[test]
fn plain_names_need_calls_for_any_prepared_replacement_or_incomplete_file() {
    let selection = plain_selection();
    let root = foreign_fixture();
    let entry = root.join("src/cross-file.mts");
    let writer = root.join("src/writer.mts");
    let files = GraphFiles::from_files(vec![entry.clone(), writer.clone()]);
    let clean = || {
        TsFactMap::from_iter_with_plan(
            [
                (entry.clone(), TsFileFacts::default()),
                (writer.clone(), TsFileFacts::default()),
            ],
            TsFactPlan::default(),
        )
    };
    assert!(!needs_effect_call_graph(&selection, &files, &clean()));

    // A write in another indexable file still keeps the canonical call graph.
    let mut foreign = clean();
    foreign.get_mut(&writer).unwrap().value_flow.replacements =
        Some(std::sync::Arc::new(Default::default()));
    assert!(needs_effect_call_graph(&selection, &files, &foreign));

    let mut nullish = clean();
    nullish
        .get_mut(&writer)
        .unwrap()
        .value_flow
        .has_nullish_alternatives = true;
    assert!(needs_effect_call_graph(&selection, &files, &nullish));

    let missing = TsFactMap::from_iter_with_plan(
        [(entry.clone(), TsFileFacts::default())],
        TsFactPlan::default(),
    );
    assert!(needs_effect_call_graph(&selection, &files, &missing));
    for failure in ["operational", "parse", "fatal"] {
        let mut facts = clean();
        let file = facts.get_mut(&writer).unwrap();
        match failure {
            "operational" => file.operational_error = Some("failed read".into()),
            "parse" => file.parse_error = Some("partial syntax".into()),
            _ => file.fatal_parse_error = true,
        }
        assert!(
            needs_effect_call_graph(&selection, &files, &facts),
            "{failure}"
        );
    }
}

#[test]
fn saved_plain_no_write_effects_keep_full_report_without_call_graph() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/effects-plain-no-write/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    // The separate predicate test proves the no-replacement plan; the public
    // query below pins the exact saved report from the matched ARM input.
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let report = crate::diagnostics::with_observer(Some(observer.clone()), || {
        run(
            fixture.path(),
            None,
            None,
            "database",
            Path::new("src/defaults.mts"),
            &[],
            None,
        )
    })
    .unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(saved.join("expected.json")).unwrap())
            .unwrap();
    assert_eq!(serde_json::to_value(report).unwrap(), expected);
    let work = observer.snapshot().work;
    // The old unconditional plain-name plan resolves all 128 consume calls:
    // 129 resolver requests and 128 cache hits for this saved physical input.
    assert_eq!(work["resolver.requests"], 1, "{work:#?}");
    assert_eq!(work.get("resolver.cache_hits").copied().unwrap_or(0), 0);
    assert_eq!(work["graph.builds"], 1);
    assert_eq!(work["parse.files"], 1);
}

#[test]
fn saved_plain_only_replacement_alternatives_and_foreign_write_survive() {
    let fixture = crate::test_support::materialize_saved_fixture(&plain_fixture());
    let actual = run(
        fixture.path(),
        None,
        None,
        "plain",
        Path::new("src/entry.mts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(actual.call_sites.len(), 1);
    let site = &actual.call_sites[0];
    assert_eq!(site.callee, "read");
    assert_eq!(
        site.alternatives
            .as_ref()
            .unwrap()
            .iter()
            .map(|value| value.callee.as_str())
            .collect::<Vec<_>>(),
        ["read", "transaction"]
    );
    let foreign = crate::test_support::materialize_saved_fixture(&foreign_fixture());
    let cross_file = run(
        foreign.path(),
        None,
        None,
        "plain",
        Path::new("src/cross-file.mts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(cross_file.call_sites.len(), 1);
    assert_eq!(
        (
            cross_file.call_sites[0].line,
            cross_file.call_sites[0].callee.as_str()
        ),
        (4, "transaction")
    );
}

#[test]
fn saved_callable_only_facts_keep_plain_report_without_receiver_call_graph() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/effects-plain-callable-only/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let path = fixture.path().join("src/entry.mts");
    let facts = crate::codebase::ts_source::facts::collect_ts_facts(
        std::slice::from_ref(&path),
        TsFactPlan {
            imports: true,
            function_calls: true,
            value_flow: true,
            ..Default::default()
        },
    );
    let flow = &facts.get(&path).unwrap().value_flow;
    let capsule = flow
        .replacements
        .as_ref()
        .expect("saved callable assignment");
    assert!(capsule.callable_only);
    assert_eq!(flow.callable_assignments().len(), 1);
    assert!(flow.callable_assignment_is_used(0, "callback"));
    assert!(!flow.callable_assignment_is_used(0, "unrelated"));
    assert!(!flow.has_receiver_replacement_facts());
    assert!(capsule.regions.is_empty() && capsule.unsafe_owners.is_empty());
    let files = GraphFiles::from_files(vec![path]);
    assert!(!needs_effect_call_graph(&plain_selection(), &files, &facts));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let actual = crate::diagnostics::with_observer(Some(observer.clone()), || {
        run(
            fixture.path(),
            None,
            None,
            "database",
            Path::new("src/entry.mts"),
            &[],
            None,
        )
    })
    .unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(saved.join("expected.json")).unwrap())
            .unwrap();
    assert_eq!(serde_json::to_value(actual).unwrap(), expected);
    let work = observer.snapshot().work;
    assert_eq!(work["parse.files"], 1);
    assert_eq!(work["resolver.requests"], 1, "{work:#?}");
    assert_eq!(work.get("resolver.cache_hits").copied().unwrap_or(0), 0);
}

#[test]
fn plain_names_keep_nullish_fallback_effects_without_replacement_facts() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/effects-plain-nullish/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let report = run(
        fixture.path(),
        None,
        None,
        "plain",
        Path::new("src/entry.mts"),
        &[],
        None,
    )
    .unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(saved.join("expected.json")).unwrap())
            .unwrap();
    assert_eq!(serde_json::to_value(&report).unwrap(), expected);
    assert_eq!(
        report
            .call_sites
            .iter()
            .map(|site| (site.line, site.callee.as_str()))
            .collect::<Vec<_>>(),
        [(5, "read"), (6, "write")]
    );
}
