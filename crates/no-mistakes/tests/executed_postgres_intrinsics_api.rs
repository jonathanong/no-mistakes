use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql_with_imports, EmbeddedSqlOptions, ExecutedPostgresSource,
    ExecutedPostgresSql, TrustedSqlTag,
};
use std::path::PathBuf;

fn extract(names: &[&str]) -> Vec<ExecutedPostgresSql> {
    let sources = names
        .iter()
        .map(|name| ExecutedPostgresSource {
            file_name: (*name).into(),
            source: std::fs::read_to_string(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../fixtures/postgres-facts/executed-source")
                    .join(name),
            )
            .unwrap(),
        })
        .collect::<Vec<_>>();
    extract_executed_postgres_sql_with_imports(
        &sources,
        &EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
            TrustedSqlTag {
                module: "@example/db".into(),
                name: "sql".into(),
            },
        ]),
        &[],
    )
    .unwrap()
}

#[test]
fn exposed_array_prototypes_block_all_supported_intrinsic_projections() {
    let rows = extract(&["control-flow-array-intrinsics.ts"]);
    assert_eq!(rows.len(), 4, "{rows:?}");
    for row in rows {
        assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
        assert!(row.sql_text.is_none(), "{row:?}");
        assert!(row.incomplete_reason.is_some(), "{row:?}");
    }
}

#[test]
fn computed_global_and_aliased_prototype_exposures_are_untrusted() {
    for name in [
        "control-flow-array-intrinsics-computed.ts",
        "control-flow-array-intrinsics-global.ts",
        "control-flow-array-intrinsics-alias.ts",
        "control-flow-array-intrinsics-exposure.ts",
        "control-flow-array-intrinsics-namespace.ts",
    ] {
        let rows = extract(&[name]);
        assert_eq!(rows.len(), 1, "{name}: {rows:?}");
        assert!(
            !rows[0].complete && !rows[0].only_value_placeholders,
            "{rows:?}"
        );
        assert_eq!(
            rows[0].incomplete_reason.as_deref(),
            Some("untrusted-array-intrinsics"),
            "{name}: {rows:?}"
        );
    }
}

#[test]
fn imported_prototype_mutations_block_future_array_projections() {
    let rows = extract(&[
        "control-flow-array-intrinsics-consumer.ts",
        "control-flow-array-intrinsics-dependency.ts",
    ]);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert!(
        !rows[0].complete && !rows[0].only_value_placeholders,
        "{rows:?}"
    );
    assert_eq!(
        rows[0].incomplete_reason.as_deref(),
        Some("untrusted-array-intrinsics")
    );
}

#[test]
fn ordinary_dense_intrinsics_remain_precise_without_prototype_exposure() {
    let rows = extract(&["control-flow-array-intrinsics-pristine.ts"]);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert!(rows[0].complete, "{rows:?}");
    assert_eq!(
        rows[0].sql_text.as_deref(),
        Some("/* intrinsics/pristine */ SELECT 1")
    );
}

#[test]
fn constructor_and_instance_aliases_taint_future_imported_intrinsics() {
    for dependency in [
        "control-flow-array-intrinsics-constructor-dependency.ts",
        "control-flow-array-intrinsics-instance-dependency.ts",
        "control-flow-array-intrinsics-derived-dependency.ts",
        "control-flow-array-intrinsics-node-dependency.ts",
    ] {
        let rows = extract(&[
            "control-flow-array-intrinsics-alias-consumer.ts",
            dependency,
        ]);
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert!(
            !rows[0].complete && !rows[0].only_value_placeholders,
            "{rows:?}"
        );
        assert_eq!(
            rows[0].incomplete_reason.as_deref(),
            Some("untrusted-array-intrinsics")
        );
    }
}

#[test]
fn reflected_array_prototypes_taint_future_imported_intrinsics() {
    let rows = [
        "control-flow-array-intrinsics-object-dependency.ts",
        "control-flow-array-intrinsics-reflect-dependency.ts",
    ]
    .map(|dependency| {
        extract(&[
            "control-flow-array-intrinsics-alias-consumer.ts",
            dependency,
        ])
    });
    assert!(
        rows.iter()
            .all(|rows| rows.len() == 1 && !rows[0].complete && !rows[0].only_value_placeholders),
        "{rows:?}"
    );
    assert!(
        rows.iter()
            .all(|rows| rows[0].incomplete_reason.as_deref() == Some("untrusted-array-intrinsics")),
        "{rows:?}"
    );
}

#[test]
fn opaque_effects_before_allocation_taint_future_intrinsics() {
    let rows = extract(&["control-flow-array-intrinsics-opaque.ts"]);
    assert_eq!(rows.len(), 7, "{rows:?}");
    for row in &rows {
        assert_eq!(row.complete, matches!(row.line, 9 | 16 | 25), "{row:?}");
    }
    assert_eq!(
        rows.iter().filter(|row| row.complete).count(),
        3,
        "{rows:?}"
    );
    for row in rows.iter().filter(|row| !row.complete) {
        assert!(!row.only_value_placeholders, "{row:?}");
        // Precomputed text was already conservatively invalidated before this fix.
        let reason = if row.line == 21 {
            "unresolved-value-or-effect"
        } else {
            "untrusted-array-intrinsics"
        };
        assert_eq!(row.incomplete_reason.as_deref(), Some(reason));
    }
    let rows = extract(&["control-flow-array-intrinsics-accessor.ts"]);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert!(
        !rows[0].complete && !rows[0].only_value_placeholders,
        "{rows:?}"
    );
    assert_eq!(
        rows[0].incomplete_reason.as_deref(),
        Some("untrusted-array-intrinsics")
    );
}
