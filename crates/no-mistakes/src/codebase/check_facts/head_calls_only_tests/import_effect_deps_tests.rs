use super::{collect_check_facts, config, plan};
use crate::diagnostics::{with_observer, InvocationObserver};

fn saved() -> tempfile::TempDir {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("import-effect-deps");
    crate::test_support::materialize_saved_fixture(&saved)
}
fn paths(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    [
        "query.mts",
        "helpers.mts",
        "leaf.mts",
        "deeper.mts",
        "reexport.mts",
        "duplicate.mts",
    ]
    .map(|name| root.join("src").join(name))
    .to_vec()
}
fn expected(source: &str) -> Vec<bool> {
    source
        .lines()
        .filter_map(|line| line.split_once("// low: "))
        .map(|(_, expected)| expected.parse().unwrap())
        .collect()
}

#[test]
fn imported_effect_obligations_match_low_mixed_and_max_prepared_facts() {
    let fixture = saved();
    let root = fixture.path();
    let paths = paths(root);
    let mut max_calls = None;
    for efforts in [vec!["max"], vec!["low"], vec!["low", "max"]] {
        let observer = InvocationObserver::new(true);
        let facts = with_observer(Some(observer.clone()), || {
            collect_check_facts(root, paths.clone(), plan(root, &config(&efforts)))
        });
        let file = facts.ts.get(&paths[0]).unwrap();
        let embedded = &file.embedded_sql[0].1;
        if efforts == ["max"] {
            assert!(!observer
                .snapshot()
                .work
                .contains_key("postgres.annotation.effect_import_censuses"));
            max_calls = Some(format!("{:?}", embedded.calls));
            continue;
        }
        for path in [&paths[0], &paths[5]] {
            let file = facts.ts.get(path).unwrap();
            let heads = file.query_annotation_heads.as_ref().unwrap();
            let source = std::fs::read_to_string(path).unwrap();
            let decisions = file.embedded_sql[0]
                .1
                .call_spans
                .iter()
                .map(|span| heads.accepts(*span))
                .collect::<Vec<_>>();
            assert_eq!(decisions, expected(&source), "{}", path.display());
        }
        if efforts == ["low"] {
            assert!(file.query_annotation.is_empty());
            assert!(embedded.calls.iter().all(|call| call.variants.is_empty()));
            assert_eq!(
                observer
                    .snapshot()
                    .work
                    .get("postgres.sql.variants_composed")
                    .copied()
                    .unwrap_or(0),
                0
            );
        } else {
            assert_eq!(Some(format!("{:?}", embedded.calls)), max_calls);
        }
    }
}

#[test]
fn imported_effect_censuses_and_export_failures_are_memoized() {
    let fixture = saved();
    let root = fixture.path();
    let caller = root.join("src/work.mts");
    let helper = root.join("src/helpers.mts");
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        collect_check_facts(
            root,
            vec![caller.clone(), helper],
            plan(root, &config(&["low"])),
        )
    });
    let file = facts.ts.get(&caller).unwrap();
    let heads = file.query_annotation_heads.as_ref().unwrap();
    let decisions = file.embedded_sql[0]
        .1
        .call_spans
        .iter()
        .map(|span| heads.accepts(*span))
        .collect::<Vec<_>>();
    assert_eq!(decisions, [true, true, false, false, false, false, false]);
    let work = observer.snapshot().work;
    // Three imported owners: positive, aliased/negative, and a missing export.
    // Repeated calls to each owner must not repeat its complete census.
    assert_eq!(
        work.get("postgres.annotation.effect_import_censuses"),
        Some(&3)
    );
    // Positive export, absent export, and the ordinary external executor lookup.
    assert_eq!(
        work.get("postgres.annotation.effect_export_lookups"),
        Some(&3)
    );
    assert_eq!(work.get("postgres.annotation.head_syntax_visits"), Some(&2));
}

#[test]
fn unprojected_local_summaries_never_publish_provisional_effect_proofs() {
    let fixture = saved();
    let root = fixture.path();
    let path = root.join("src/work.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let plan = plan(root, &config(&["low"]));
    let file = crate::ast::with_program(&path, &source, |program, source| {
        crate::codebase::check_facts::collect_file_facts_from_program(
            root, &path, &plan, None, source, program, None,
        )
    })
    .unwrap();
    let heads = file.query_annotation_heads.as_ref().unwrap();
    assert!(file.embedded_sql[0]
        .1
        .call_spans
        .iter()
        .all(|span| !heads.accepts(*span)));
    assert!(heads
        .pending_effects
        .iter()
        .any(|pending| pending.require_plain));
    assert!(!heads.exported_effects.contains_key("positive"));
}

#[path = "import_effect_integration_tests.rs"]
mod integration;
