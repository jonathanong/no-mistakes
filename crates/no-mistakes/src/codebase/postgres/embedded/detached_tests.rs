use super::{extract_embedded_sql_from_source, EmbeddedSqlKind, EmbeddedSqlOptions};
use std::path::PathBuf;

fn facts(name: &str, options: &EmbeddedSqlOptions) -> super::EmbeddedSqlFileFacts {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-test-state-mutation/detached-executors/tests")
        .join(name);
    extract_embedded_sql_from_source(&path, &std::fs::read_to_string(&path).unwrap(), options)
}
fn profile() -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured("@example/db", &[])
}

#[test]
fn detached_shapes_keep_exact_sql_and_original_capture_time() {
    for (file, lines) in [
        ("exact.test.ts", vec![4, 7, 10, 12, 14, 17, 19]),
        ("shape-forms.test.ts", vec![3, 5, 8, 10]),
        ("after-copy.test.ts", vec![7]),
        ("scopes.test.ts", vec![3, 8]),
        ("receiver-escaped-after.test.ts", vec![6]),
    ] {
        let facts = facts(file, &profile());
        assert_eq!(facts.calls.len(), lines.len(), "{file}: {facts:?}");
        for line in lines {
            let call = facts.calls.iter().find(|call| call.line == line).unwrap();
            assert_eq!(call.kind, EmbeddedSqlKind::Inline, "{file}:{line}");
            assert!(call.sql_text.is_some(), "{file}:{line}");
        }
    }
}

#[test]
fn called_bodies_retain_exact_and_uncertain_detached_executors() {
    let facts = facts("local-bodies.test.ts", &profile());
    assert!(facts
        .calls
        .iter()
        .any(|call| call.line == 5 && call.kind == EmbeddedSqlKind::Inline));
    assert!(facts
        .calls
        .iter()
        .any(|call| call.line == 11 && call.is_unanalyzable()));
}

#[test]
fn incomplete_detachment_does_not_certify_caller_sql() {
    for (file, line) in [
        ("mixed-roles.test.ts", 5),
        ("mixed-roles.test.ts", 6),
        ("mixed-roles.test.ts", 9),
        ("mixed-roles.test.ts", 10),
        ("assigned.test.ts", 4),
        ("assigned.test.ts", 7),
        ("assigned.test.ts", 10),
        ("assigned.test.ts", 14),
        ("assigned.test.ts", 17),
        ("assigned.test.ts", 20),
        ("assigned.test.ts", 22),
        ("local-bodies.test.ts", 11),
        ("wrong-this.test.ts", 3),
        ("prebound.test.ts", 5),
        ("mutable.test.ts", 6),
        ("uncertain.test.ts", 7),
        ("escaped.test.ts", 5),
        ("receiver-escaped.test.ts", 6),
        ("receiver-history.test.ts", 8),
        ("receiver-origins.test.ts", 8),
        ("receiver-origins.test.ts", 10),
        ("uncertain-shapes.test.ts", 4),
        ("uncertain-shapes.test.ts", 6),
    ] {
        let facts = facts(file, &profile());
        let call = facts.calls.iter().find(|call| call.line == line).unwrap();
        assert_eq!(call.kind, EmbeddedSqlKind::Dynamic, "{file}:{line}");
        assert!(call.sql_text.is_none() && call.variants.is_empty());
    }
}

#[test]
fn detached_executors_require_exact_runtime_imports_and_query_selection() {
    assert!(facts("controls.test.ts", &profile()).calls.is_empty());
    let query_only = EmbeddedSqlOptions::configured("@example/db", &["query".into()]);
    assert!(facts("callable-controls.test.ts", &query_only)
        .calls
        .is_empty());
    let custom =
        EmbeddedSqlOptions::configured("@example/db", &["query".into(), "customExecutor".into()])
            .with_scoped_executors(&["start".into()], &[]);
    assert!(facts("../callable-profile.fixture.ts", &custom)
        .calls
        .is_empty());
    for options in [
        EmbeddedSqlOptions::configured("@other/db", &[]),
        EmbeddedSqlOptions::configured("@example/db", &["execute".into()]),
    ] {
        assert!(facts("exact.test.ts", &options).calls.is_empty());
    }
}

#[test]
fn imported_receiver_members_and_conditionals_keep_proof_boundaries() {
    let facts = facts("../receiver-proofs.fixture.ts", &profile());
    for line in [4, 6, 8, 10] {
        assert!(
            facts
                .calls
                .iter()
                .any(|call| call.line == line && call.kind == EmbeddedSqlKind::Inline),
            "{line}: {facts:?}"
        );
    }
    for line in [13, 16, 19] {
        assert!(
            facts
                .calls
                .iter()
                .any(|call| call.line == line && call.is_unanalyzable()),
            "{line}: {facts:?}"
        );
    }
}
