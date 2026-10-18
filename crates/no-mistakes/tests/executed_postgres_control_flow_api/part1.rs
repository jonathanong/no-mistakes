use super::*;

#[test]
fn whole_statement_helper_branches_and_collections() {
    let rows = extract("control-flow-collections.ts");
    for (annotation, count) in [
        ("branch/read", 2),
        ("collections/map", 1),
        ("collections/conditional", 2),
        ("collections/append", 1),
        ("collections/empty-append", 1),
        ("branch/mixed", 2),
    ] {
        let variants: Vec<_> = rows
            .iter()
            .filter(|row| row.annotation.as_deref() == Some(annotation))
            .collect();
        assert_eq!(variants.len(), count, "{annotation}: {rows:?}");
        for variant in variants {
            assert!(
                variant.complete || variant.only_value_placeholders,
                "{variant:?}"
            );
            assert!(variant.incomplete_reason.is_none(), "{variant:?}");
            assert!(
                parse_postgres_sql(variant.sql_text.as_ref().unwrap()).is_ok(),
                "{variant:?}"
            );
        }
    }
}

#[test]
fn unknown_runtime_collections_and_effects_never_prove_reads() {
    let rows = extract("control-flow-collections.ts");
    let unknown: Vec<_> = rows
        .iter()
        .filter(|row| !row.complete && !row.only_value_placeholders)
        .collect();
    assert_eq!(unknown.len(), 4, "{rows:?}");
    for row in unknown {
        assert!(row.incomplete_reason.is_some(), "{row:?}");
    }
}

#[test]
fn shared_variant_limit_and_every_alternative_are_required() {
    let rows = extract("control-flow-limits.ts");
    let at_limit: Vec<_> = rows
        .iter()
        .filter(|row| row.annotation.as_deref() == Some("variants/at-limit"))
        .collect();
    assert_eq!(
        at_limit.len(),
        no_mistakes::codebase::postgres::MAX_EMBEDDED_SQL_VARIANTS
    );
    assert!(at_limit.iter().all(|row| row.complete));
    assert!(rows
        .iter()
        .any(|row| row.incomplete_reason.as_deref() == Some("variant-limit")));
    for row in rows.iter().filter(|row| {
        row.annotation.as_deref() != Some("variants/at-limit")
            && row.annotation.as_deref() != Some("append/mixed")
    }) {
        assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
        assert!(row.incomplete_reason.is_some(), "{row:?}");
    }
    let mixed: Vec<_> = rows
        .iter()
        .filter(|row| row.annotation.as_deref() == Some("append/mixed"))
        .collect();
    assert_eq!(mixed.len(), 2, "{rows:?}");
    assert!(mixed.iter().all(|row| row.complete
        && row
            .sql_text
            .as_ref()
            .unwrap()
            .contains("SELECT id FROM removed")));
    assert!(mixed.iter().any(|row| row
        .sql_text
        .as_ref()
        .unwrap()
        .contains("DELETE FROM records")));
}
