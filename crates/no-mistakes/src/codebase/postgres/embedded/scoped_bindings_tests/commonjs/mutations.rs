use super::*;

#[test]
fn factory_member_mutations_keep_calls_but_only_callable_sql_proof() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&["beginTransaction".into()], &[]);
    let mut failures = Vec::new();
    for (name, sql) in [
        (
            "factory-member-query-assignment.ts",
            "SELECT direct assignment",
        ),
        ("factory-member-query-delete.ts", "SELECT direct delete"),
        ("factory-member-query-computed.ts", "SELECT direct computed"),
        ("factory-member-query-escape.ts", "SELECT direct escape"),
        ("factory-mutable-alias.ts", "SELECT direct mutable alias"),
        (
            "factory-container-escape.ts",
            "SELECT direct container escape",
        ),
        (
            "factory-late-mutable-alias.ts",
            "SELECT direct late mutable",
        ),
        (
            "factory-conditional-alias.ts",
            "SELECT direct conditional alias",
        ),
        (
            "factory-conditional-container.ts",
            "SELECT direct conditional container",
        ),
        ("factory-return-escape.ts", "SELECT direct return escape"),
        (
            "factory-conditional-argument.ts",
            "SELECT direct conditional argument",
        ),
        (
            "factory-arrow-return-escape.ts",
            "SELECT direct arrow escape",
        ),
        (
            "factory-sequence-escape.ts",
            "SELECT direct sequence escape",
        ),
        (
            "factory-const-conditional-alias.ts",
            "SELECT direct const conditional",
        ),
        (
            "factory-const-logical-alias.ts",
            "SELECT direct const logical",
        ),
        (
            "factory-const-sequence-alias.ts",
            "SELECT direct const sequence",
        ),
        (
            "factory-const-mixed-namespace-alias.ts",
            "SELECT direct const mixed namespace",
        ),
    ] {
        let path = fixture(name);
        let source = std::fs::read_to_string(&path).expect("saved factory mutation fixture");
        let facts = extract_embedded_sql_from_source(&path, &source, &options);
        let direct = facts
            .calls
            .iter()
            .filter(|call| call.callee != "query")
            .collect::<Vec<_>>();
        let members = facts
            .calls
            .iter()
            .filter(|call| call.callee == "query")
            .collect::<Vec<_>>();
        if direct.len() != 1
            || direct[0].sql_text.as_deref() != Some(sql)
            || members.is_empty()
            || !members.iter().all(|call| call.is_unanalyzable())
        {
            failures.push((name, facts.calls));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
