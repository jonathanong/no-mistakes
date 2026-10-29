use super::{collect_check_facts, config, plan};
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::diagnostics::{with_observer, InvocationObserver};

#[test]
fn conditional_effect_groups_share_each_exact_builders_lifetime_requirements() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("conditional-effect-groups");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let caller = root.join("src/query.mts");
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        collect_check_facts(
            root,
            vec![caller.clone(), root.join("src/helpers.mts")],
            plan(root, &config(&["low"])),
        )
    });
    let file = facts.ts.get(&caller).unwrap();
    assert!(file.query_annotation.is_empty());
    let heads = file.query_annotation_heads.as_ref().unwrap();
    let source = std::fs::read_to_string(&caller).unwrap();
    let mut accepted = 0;
    let mut rejected = 0;
    for (call, span) in file.embedded_sql[0]
        .1
        .calls
        .iter()
        .zip(&file.embedded_sql[0].1.call_spans)
    {
        let line = source.lines().nth(call.line as usize - 1).unwrap();
        let expected = if line.contains("// accepted") {
            accepted += 1;
            HeadProof::Literal
        } else {
            assert!(line.contains("// unanalyzable"), "{line}");
            rejected += 1;
            HeadProof::Unproven
        };
        assert_eq!(heads.calls.get(span).copied(), Some(expected), "{line}");
    }
    assert_eq!((accepted, rejected), (16, 16));
    let owners = file
        .ts
        .module_bindings
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .filter(|owner| owner.name == "q")
        .collect::<Vec<_>>();
    assert_eq!(owners.len(), 2);
    assert_ne!(owners[0].id, owners[1].id);
    assert_ne!(owners[0].scope_id, owners[1].scope_id);
    let work = observer.snapshot().work;
    for (metric, expected) in [
        ("postgres.annotation.conditional_effect_normalizations", 2),
        ("postgres.annotation.conditional_effect_groups", 2),
        (
            "postgres.annotation.conditional_effect_requirements_copied",
            16,
        ),
        ("postgres.annotation.conditional_effect_approval_checks", 16),
    ] {
        assert_eq!(
            work.get(metric).copied(),
            Some(expected),
            "{metric}: {work:#?}"
        );
    }
    assert_eq!(
        work.get("postgres.sql.variants_composed")
            .copied()
            .unwrap_or(0),
        0
    );
    assert_eq!(heads.conditional_effects.len(), 2);
}
