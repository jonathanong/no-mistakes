use super::super::shape_tests::{config_yaml, fixture};
use super::super::*;
use crate::codebase::postgres::embedded::{extract_embedded_sql_from_source, EmbeddedSqlOptions};

#[test]
fn callable_binding_await_and_lexical_shadow_keep_only_reached_returns() {
    let root = fixture("returned-call-identities");
    for (name, expected) in [
        ("named", 3),
        ("await", 3),
        ("shadow", 5),
        ("recursive", 6),
        ("shadow-edge", 5),
    ] {
        let path = root.join(format!("src/{name}.ts"));
        let source = std::fs::read_to_string(&path).unwrap();
        let facts = extract_embedded_sql_from_source(
            &path,
            &source,
            &EmbeddedSqlOptions::configured("@example/db", &[]),
        );
        assert_eq!(
            facts
                .fragments
                .iter()
                .map(|fragment| fragment.line)
                .collect::<Vec<_>>(),
            [expected],
            "{name}: {facts:#?}"
        );
        let findings = check_with_files(
            &root,
            &config_yaml("importSpecifier: '@example/db'\nunanalyzableSql: ignore\nbannedShapes: [literal-limit]"),
            &[path],
        ).unwrap();
        assert_eq!(
            findings
                .iter()
                .map(|finding| (finding.target.as_deref(), finding.line))
                .collect::<Vec<_>>(),
            [(Some("literal-limit"), expected as usize)],
            "{name}: {findings:#?}"
        );
    }
}

#[test]
fn shadowed_call_never_reports_unused_outer_non_sql_return() {
    let root = fixture("returned-call-identities");
    let findings = check_with_files(
        &root,
        &config_yaml(
            "importSpecifier: '@example/db'\nunanalyzableSql: fail\nbannedShapes: [literal-limit]",
        ),
        &[root.join("src/shadow.ts")],
    )
    .unwrap();
    // The unresolved executor remains fail-closed, but the unused outer return
    // must not add its own non-SQL finding at line 3.
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [5, 6],
        "{findings:#?}"
    );
}

#[test]
fn unkeyable_member_returns_keep_conservative_recall() {
    let root = fixture("returned-member-owners");
    for name in [
        "class-method",
        "this-method",
        "object-method",
        "property-function",
        "named-property-function",
        "unused-method",
    ] {
        let path = root.join(format!("src/{name}.ts"));
        let source = std::fs::read_to_string(&path).unwrap();
        let facts = extract_embedded_sql_from_source(
            &path,
            &source,
            &EmbeddedSqlOptions::configured("@example/db", &[]),
        );
        // Member owners are conservative; the unused lexical helpers stay filtered.
        assert_eq!(
            facts
                .fragments
                .iter()
                .map(|fragment| fragment.line)
                .collect::<Vec<_>>(),
            [4],
            "{name}: {facts:#?}",
        );
        let findings = check_with_files(
            &root,
            &serde_yaml::from_str(
                &std::fs::read_to_string(root.join(format!("{name}.yml"))).unwrap(),
            )
            .unwrap(),
            &[path],
        )
        .unwrap();
        assert_eq!(
            findings
                .iter()
                .map(|finding| (finding.target.as_deref(), finding.line))
                .collect::<Vec<_>>(),
            [(Some("literal-limit"), 4)],
            "{name}: {findings:#?}",
        );
    }
}
