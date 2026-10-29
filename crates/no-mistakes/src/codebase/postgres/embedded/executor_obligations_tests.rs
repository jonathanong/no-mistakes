use super::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use std::path::PathBuf;

#[test]
fn every_non_direct_executor_reference_retains_an_unknown_obligation() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-test-state-mutation/detached-obligations/tests");
    let options = EmbeddedSqlOptions::configured(
        "@example/db",
        &[
            "query".into(),
            "read".into(),
            "write".into(),
            "execute".into(),
        ],
    );
    for (name, lines) in [
        ("references.test.ts", vec![3, 5, 6, 7, 12, 15, 23]),
        ("names.test.ts", vec![3, 5, 8, 10]),
        ("parameter-alias.test.ts", vec![3, 7]),
    ] {
        let path = root.join(name);
        let facts = extract_embedded_sql_from_source(
            &path,
            &std::fs::read_to_string(&path).unwrap(),
            &options,
        );
        for line in lines {
            assert!(
                facts
                    .calls
                    .iter()
                    .any(|call| call.line == line && call.is_unanalyzable()),
                "{name}:{line}: {facts:?}"
            );
        }
    }
    // Mentioning an executor without invoking it is not an obligation.
    for (name, lines) in [
        (
            "references.test.ts",
            vec![4, 8, 9, 10, 11, 14, 19, 20, 21, 22, 24],
        ),
        ("names.test.ts", vec![4, 6, 7, 9]),
        ("forward-export.test.ts", vec![2, 3]),
        ("parameter-alias.test.ts", vec![2, 6]),
    ] {
        let path = root.join(name);
        let facts = extract_embedded_sql_from_source(
            &path,
            &std::fs::read_to_string(&path).unwrap(),
            &options,
        );
        for line in lines {
            assert!(
                !facts.calls.iter().any(|call| call.line == line),
                "{name}:{line}: {facts:?}"
            );
        }
    }
    let path = root.join("direct-wrappers.test.ts");
    let wrappers =
        extract_embedded_sql_from_source(&path, &std::fs::read_to_string(&path).unwrap(), &options);
    for line in [2, 4] {
        let calls = wrappers
            .calls
            .iter()
            .filter(|call| call.line == line)
            .collect::<Vec<_>>();
        assert_eq!(calls.len(), 1, "{wrappers:?}");
        assert!(calls[0].sql_text.is_some(), "{wrappers:?}");
    }
    let path = root.join("wrapped-reference.test.ts");
    let wrapped =
        extract_embedded_sql_from_source(&path, &std::fs::read_to_string(&path).unwrap(), &options);
    assert_eq!(
        wrapped.calls.iter().filter(|call| call.line == 2).count(),
        0
    );
    let path = root.join("known.test.ts");
    let custom = extract_embedded_sql_from_source(
        &path,
        &std::fs::read_to_string(&path).unwrap(),
        &EmbeddedSqlOptions::configured("@example/db", &["execute".into()]),
    );
    assert_eq!(custom.calls.len(), 1, "{custom:?}");
    assert_eq!(custom.calls[0].line, 6);
    assert!(custom.calls[0].sql_text.is_some());
    let defaults = extract_embedded_sql_from_source(
        &path,
        &std::fs::read_to_string(&path).unwrap(),
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    for line in [3, 4, 5] {
        assert!(
            defaults
                .calls
                .iter()
                .any(|call| call.line == line && call.sql_text.is_some()),
            "{defaults:?}"
        );
    }
    assert!(!defaults.calls.iter().any(|call| call.line == 6));
    for name in ["controls.test.ts", "type-uses.test.ts"] {
        let path = root.join(name);
        assert!(
            extract_embedded_sql_from_source(
                &path,
                &std::fs::read_to_string(&path).unwrap(),
                &options,
            )
            .calls
            .is_empty(),
            "{name}"
        );
    }
}
