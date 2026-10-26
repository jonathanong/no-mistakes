use crate::codebase::postgres::{EmbeddedSqlOptions, TrustedSqlTag};
use std::path::PathBuf;

#[test]
fn executed_fragment_filter_matches_contributors_and_physical_sql_origins() {
    let profile = EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
        TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        },
    ]);
    for (name, expected_lines, receiver) in [
        ("executed-complete-reuse", vec![2, 3], "unrelated.append"),
        ("executed-complete-owned", vec![2, 3], "unrelated.append"),
        ("executed-complete-origin", vec![3], "own.append"),
        ("executed-complete-helper-owned", vec![], "input.append"),
        (
            "executed-complete-helper-reuse",
            vec![9, 4],
            "unrelated.append",
        ),
    ] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../test-cases/rules/postgres-sql-shape-policy/variants/{name}/src/query.ts",
        ));
        let source = std::fs::read_to_string(&path).unwrap();
        let facts =
            crate::codebase::postgres::extract_embedded_sql_from_source(&path, &source, &profile);
        assert_eq!(facts.calls.len(), 1, "{name}");
        assert_eq!(facts.calls[0].variants.len(), 2, "{name}");
        let prepared = super::super::collect(&facts);
        assert_eq!(
            prepared
                .iter()
                .map(|fragment| fragment.line)
                .collect::<Vec<_>>(),
            expected_lines,
            "{name}"
        );
        let site = source.find(receiver).unwrap() as u32;
        assert!(
            prepared
                .iter()
                .all(|fragment| fragment.append_site == Some(site)),
            "{name}"
        );
        assert!(
            prepared
                .iter()
                .all(|fragment| !fragment.statements.parse_failed),
            "{name}"
        );
    }
}
