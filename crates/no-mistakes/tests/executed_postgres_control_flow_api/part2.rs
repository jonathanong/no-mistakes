use super::*;

#[test]
fn read_classification_requires_all_parsed_variants_and_rejects_nested_effects() {
    let rows = extract("control-flow-collections.ts");
    let group = |annotation| {
        rows.iter()
            .filter(|row| row.annotation.as_deref() == Some(annotation))
            .collect::<Vec<_>>()
    };
    assert!(read_statement_shape(&group("branch/read")));
    assert!(read_statement_shape(&group("collections/map")));
    assert!(!read_statement_shape(&group("branch/mixed")));
    let rows = extract("control-flow-limits.ts");
    let mixed = rows
        .iter()
        .filter(|row| row.annotation.as_deref() == Some("append/mixed"))
        .collect::<Vec<_>>();
    assert!(!read_statement_shape(&mixed));
    assert!(!read_statement_shape(
        &rows.iter().filter(|row| !row.complete).collect::<Vec<_>>()
    ));
}

#[test]
fn opaque_collection_callbacks_and_shapes_keep_explicit_blockers() {
    let rows = extract("control-flow-effects.ts");
    for row in rows.iter().filter(|row| {
        !matches!(
            row.annotation.as_deref(),
            Some(
                "effects/live-elements"
                    | "effects/comparisons"
                    | "effects/mapped-branches"
                    | "effects/unrelated-deletion"
                    | "effects/false-branch"
            )
        )
    }) {
        assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
        assert!(row.incomplete_reason.is_some(), "{row:?}");
    }
    let live = rows
        .iter()
        .find(|row| row.annotation.as_deref() == Some("effects/live-elements"))
        .unwrap();
    assert!(live.complete, "{live:?}");
    assert!(live
        .sql_text
        .as_ref()
        .unwrap()
        .contains("DELETE FROM records"));
    assert!(!read_statement_shape(&[live]));
    let comparisons = rows
        .iter()
        .find(|row| row.annotation.as_deref() == Some("effects/comparisons"))
        .unwrap();
    assert_eq!(
        comparisons.sql_text.as_deref(),
        Some("/* effects/comparisons */ SELECT 1,2")
    );
}

#[test]
fn mapped_element_alternatives_produce_whole_statement_variants() {
    let rows = extract("control-flow-effects.ts");
    let variants = rows
        .iter()
        .filter(|row| row.annotation.as_deref() == Some("effects/mapped-branches"))
        .collect::<Vec<_>>();
    assert_eq!(variants.len(), 4, "{rows:?}");
    assert!(read_statement_shape(&variants));
}

#[test]
fn collection_deletion_is_scoped_and_static_false_branches_do_not_invent_writes() {
    let rows = extract("control-flow-effects.ts");
    for name in ["effects/unrelated-deletion", "effects/false-branch"] {
        let variants = rows
            .iter()
            .filter(|row| row.annotation.as_deref() == Some(name))
            .collect::<Vec<_>>();
        assert_eq!(variants.len(), 1, "{rows:?}");
        assert!(read_statement_shape(&variants), "{variants:?}");
    }
}

#[test]
fn opaque_operand_effects_survive_final_collection_and_scalar_values() {
    let rows = extract("control-flow-operand-effects.ts");
    assert_eq!(rows.len(), 14, "{rows:?}");
    let safe = rows
        .iter()
        .filter(|row| row.complete || row.only_value_placeholders)
        .collect::<Vec<_>>();
    assert_eq!(safe.len(), 2, "{rows:?}");
    assert!(read_statement_shape(&safe));
    for row in rows
        .iter()
        .filter(|row| !row.complete && !row.only_value_placeholders)
    {
        assert!(row.sql_text.is_none(), "{row:?}");
        assert!(row.incomplete_reason.is_some(), "{row:?}");
    }
}

#[test]
fn sparse_shapes_do_not_choose_safe_branches_and_literal_elements_refresh_builders() {
    let rows = extract("control-flow-sparse-snapshots.ts");
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert!(
        !rows[0].complete && !rows[0].only_value_placeholders,
        "{rows:?}"
    );
    assert_eq!(
        rows[0].incomplete_reason.as_deref(),
        Some("sparse-or-opaque-collection-shape")
    );
    assert!(rows[1].complete, "{rows:?}");
    assert!(rows[1]
        .sql_text
        .as_ref()
        .unwrap()
        .contains("DELETE FROM records"));
    assert!(!read_statement_shape(&[&rows[1]]));
}
