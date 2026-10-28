use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn async_export_aliases_memoize_positive_and_negative_owner_censuses() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("async-return-work");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let caller = root.join("src/query.mts");
    let helper = root.join("src/helpers.mts");
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        collect_check_facts(
            root,
            vec![caller.clone(), helper.clone()],
            plan(root, &config(&["low"])),
        )
    });
    let file = facts.ts.get(&helper).unwrap();
    let module = file.ts.module_bindings.as_ref().unwrap();
    let callable_visits = module
        .bindings
        .iter()
        .filter(|binding| matches!(binding.name.as_str(), "native" | "custom"))
        .map(|binding| binding.references.len())
        .sum::<usize>();
    let builder_visits = module
        .bindings
        .iter()
        .filter(|binding| binding.name == "nativeQuery")
        .map(|binding| binding.references.iter().filter(|u| u.runtime).count())
        .sum::<usize>();
    assert!(callable_visits >= 32);
    let work = observer.snapshot().work;
    assert_eq!(
        work.get("postgres.annotation.async_return_censuses")
            .copied(),
        Some(2)
    );
    assert_eq!(
        work.get("postgres.annotation.async_return_reference_visits")
            .copied(),
        Some((callable_visits + builder_visits) as u64)
    );
    let expected_export_visits = [&caller, &helper]
        .into_iter()
        .map(|path| {
            facts
                .ts
                .get(path)
                .unwrap()
                .ts
                .module_bindings
                .as_ref()
                .unwrap()
                .exports
                .iter()
                .filter(|export| !export.type_only)
                .count()
        })
        .sum::<usize>();
    assert!(expected_export_visits >= 80);
    assert_eq!(
        work.get("postgres.annotation.async_export_index_visits")
            .copied(),
        Some(expected_export_visits as u64)
    );
    let expected_import_visits = facts
        .ts
        .get(&caller)
        .unwrap()
        .ts
        .module_bindings
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .filter(|b| b.imported && (b.name.starts_with("native") || b.name.starts_with("custom")))
        .map(|b| b.references.len())
        .sum::<usize>();
    assert_eq!(
        work.get("postgres.annotation.async_import_censuses")
            .copied(),
        Some(16)
    );
    assert_eq!(
        work.get("postgres.annotation.async_import_reference_visits")
            .copied(),
        Some(expected_import_visits as u64)
    );
    let heads = facts
        .ts
        .get(&caller)
        .unwrap()
        .query_annotation_heads
        .as_ref()
        .unwrap();
    let proofs = heads
        .calls
        .iter()
        .filter(|(span, _)| {
            // Match only confirmed executor spans, excluding the awaited factory calls.
            facts.ts.get(&caller).unwrap().embedded_sql[0]
                .1
                .call_spans
                .contains(span)
        })
        .map(|(_, proof)| *proof)
        .collect::<Vec<_>>();
    assert_eq!(&proofs[..16], &[HeadProof::Literal; 16]);
    assert_eq!(&proofs[16..], &[HeadProof::Unproven; 8]);
}

#[test]
fn ordinary_awaited_calls_do_not_construct_async_origin_censuses() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("async-return-origin");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        collect_check_facts(
            root,
            vec![root.join("src/ordinary.mts")],
            plan(root, &config(&["low"])),
        )
    });
    let work = observer.snapshot().work;
    assert!(!work.contains_key("postgres.annotation.async_return_censuses"));
    assert!(!work.contains_key("postgres.annotation.async_return_reference_visits"));
}
