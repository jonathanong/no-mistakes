use super::collect;
use crate::codebase::{
    dependencies::extract::CallableId,
    postgres::embedded::walk::resolve::functions::provenance::RecoveredSql,
    ts_source::facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;

fn fixture(file: &str) -> (String, FxHashMap<CallableId, Vec<RecoveredSql>>) {
    let path = fixture_root().join(file);
    let source = std::fs::read_to_string(&path).unwrap();
    let sites = crate::ast::with_program(&path, &source, |program, _| {
        let ts = collect_file_facts_from_program(
            &path,
            TsFactPlan {
                module_bindings: true,
                ..Default::default()
            },
            &TsFactContext::default(),
            &source,
            program,
            None,
            None,
        );
        let Some(module) = ts.module_bindings.as_deref() else {
            return FxHashMap::default();
        };
        collect(program, module, &source)
    })
    .unwrap();
    (source, sites)
}

fn fixture_root() -> PathBuf {
    crate::test_support::rule_fixture_root("postgres-no-offset").join("low-helper-recovery/src")
}

fn edge_fixture(file: &str) -> (String, FxHashMap<CallableId, Vec<RecoveredSql>>) {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("variants/coverage-proof-edges")
        .join(file);
    let source = std::fs::read_to_string(&path).unwrap();
    let sites = crate::ast::with_program(&path, &source, |program, _| {
        let ts = collect_file_facts_from_program(
            &path,
            TsFactPlan {
                module_bindings: true,
                ..Default::default()
            },
            &TsFactContext::default(),
            &source,
            program,
            None,
            None,
        );
        collect(program, ts.module_bindings.as_deref().unwrap(), &source)
    })
    .unwrap();
    (source, sites)
}

#[test]
fn collects_rows_for_a_named_vitest_runner_and_inline_block_arrow() {
    let (source, sites) = fixture("iterator-positive.mts");
    assert_eq!(sites.len(), 1);
    let (callback, rows) = sites.iter().next().unwrap();
    assert!((rows[0].origins[0] as usize) < callback.0 as usize);
    assert_eq!(
        rows.iter().map(|row| row.sql.as_str()).collect::<Vec<_>>(),
        ["placement_id", "image_id", "binding_family"]
    );
    assert!(rows.iter().all(|row| row.origins.len() == row.sql.len()));
    assert!(
        rows.windows(2)
            .all(|pair| pair[0].origins[0] < pair[1].origins[0]),
        "row origins must preserve source order"
    );
    for row in rows {
        assert_eq!(
            &source[row.origins[0] as usize..row.origins[0] as usize + row.sql.len()],
            row.sql
        );
    }
}

#[test]
fn rejects_unbounded_rows_aliases_and_mutable_runner_methods() {
    for file in [
        "iterator-dynamic.mts",
        "iterator-alias.mts",
        "iterator-mutation.mts",
        "iterator-namespace-mutation.mts",
    ] {
        let (_, sites) = fixture(file);
        assert!(sites.is_empty(), "{file}");
    }
}

#[test]
fn only_inline_literal_rows_and_plain_block_callbacks_become_finite_inputs() {
    let (source, sites) = edge_fixture("iterator-wrapped.mts");
    let mut rows = sites
        .values()
        .flat_map(|values| values.iter().map(|value| value.sql.as_str()))
        .collect::<Vec<_>>();
    rows.sort_unstable();
    assert_eq!(rows, ["non-null", "wrapped"]);
    for values in sites.values() {
        for value in values {
            assert_eq!(
                &source[value.origins[0] as usize..value.origins[0] as usize + value.sql.len()],
                value.sql
            );
        }
    }
}

#[test]
fn unsupported_runner_uses_invalidate_the_whole_file() {
    let (_, sites) = edge_fixture("iterator-shapes.mts");
    assert!(sites.is_empty());
}

#[test]
fn direct_eval_invalidates_even_otherwise_finite_rows() {
    let (_, sites) = edge_fixture("iterator-eval.mts");
    assert!(sites.is_empty());
}

#[test]
fn dynamic_with_scope_invalidates_runner_proofs() {
    use oxc_ast_visit::Visit;

    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("variants/coverage-proof-edges/iterator-with.cjs");
    let source = std::fs::read_to_string(&path).unwrap();
    let unsafe_file = crate::ast::with_program(&path, &source, |program, _| {
        let mut collector = super::collector::Collector {
            source: &source,
            slots: FxHashMap::default(),
            runners: FxHashSet::default(),
            sites: FxHashMap::default(),
            runner_uses: FxHashSet::default(),
            unsafe_file: false,
        };
        collector.visit_program(program);
        collector.unsafe_file
    })
    .unwrap();
    assert!(unsafe_file);
}
