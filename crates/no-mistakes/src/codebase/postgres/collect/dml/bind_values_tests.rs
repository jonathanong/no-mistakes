use crate::codebase::postgres::{
    extract_embedded_sql_from_source, EmbeddedSqlOptions, SqlBoundItemKind, SqlPinSource,
    TrustedSqlTag,
};

#[test]
fn repeated_primitive_bindings_keep_private_identity_through_bound_extraction() {
    let root = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("../issue-3151-embedded/fixture");
    let path = root.join("repeated-binding.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@example/db", &["query".into()])
        .with_trusted_sql_tags(&[TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        }]);
    let embedded = extract_embedded_sql_from_source(&path, &source, &options);
    let [call] = embedded.calls.as_slice() else {
        panic!("one saved executor call");
    };
    assert_eq!(call.bound_value_origins.len(), 2, "{call:#?}");
    let declaration = source.find("userId: number").unwrap() as u32;
    assert!(call
        .bound_value_origins
        .iter()
        .all(|(_, _, id)| *id == declaration));
    assert_eq!(call.bound_value_sql, call.sql_text);
    let sql = call.sql_text.as_deref().unwrap();
    assert!(sql.contains("m.user_id >= sql_placeholder_2"));
    assert!(sql.contains("m.user_id <= sql_placeholder_3"));
    let binds = super::collect(call, &path);
    assert_eq!(binds.len(), 2, "{call:#?}");
    assert!(binds.values().all(|id| *id == declaration));
    let facts = crate::codebase::postgres::collect::dml::embedded_call_facts_with_json_writes(
        &embedded,
        true,
        false,
        crate::codebase::postgres::statement_facts::SqlLocationDemand::None,
        false,
        false,
        true,
    );
    let member = facts
        .iter()
        .flat_map(|file| &file.bounds)
        .flat_map(|bound| &bound.query.items)
        .find(|item| matches!(&item.kind, SqlBoundItemKind::Table(name) if name == "members"))
        .expect("saved scalar relation");
    assert!(member
        .pins
        .iter()
        .any(|pin| pin.column == "group_id" && matches!(pin.source, SqlPinSource::Value)));
    assert!(
        member
            .pins
            .iter()
            .any(|pin| pin.column == "user_id" && matches!(pin.source, SqlPinSource::Value)),
        "{member:#?}"
    );
}

#[test]
fn repeated_parameter_fields_keep_distinct_sql_placeholders_and_shared_identity() {
    let root = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("../issue-3151-real/fixture");
    let path = root.join("member-closed-range.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@example/db", &["query".into()]);
    let embedded = extract_embedded_sql_from_source(&path, &source, &options);
    let [call] = embedded.calls.as_slice() else {
        panic!("one saved executor call");
    };
    let sql = call.sql_text.as_deref().unwrap();
    assert!(sql.contains("m.user_id >= sql_placeholder_2"));
    assert!(sql.contains("m.user_id <= sql_placeholder_3"));
    assert_eq!(call.bound_value_sql, call.sql_text);
    let binds = super::collect(call, &path);
    assert_eq!(binds.len(), 4, "{call:#?}");
    let mut identities: Vec<_> = binds.values().copied().collect();
    identities.sort_unstable();
    assert_eq!(identities[0], identities[1]);
    assert_eq!(identities[2], identities[3]);
    assert_ne!(identities[0], identities[2]);
}

#[test]
fn member_identity_requires_the_declared_contract_or_an_unescaped_own_data_field() {
    let root = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("../issue-3151-real/fixture");
    let options = EmbeddedSqlOptions::configured("@example/db", &["query".into()]);
    for (name, stable) in [
        ("closed-range", true),
        ("own-data", true),
        ("alias-control", false),
        ("computed-control", false),
        ("constructor-control", false),
        ("direct-mutation-control", false),
        ("distinct-control", false),
        ("external-call-control", false),
        ("external-tag-control", false),
        ("getter-control", false),
        ("mutation-control", false),
        ("own-data-escape-control", false),
        ("unknown-type", true),
        ("volatile-control", false),
        ("proxy-control", false),
        ("accessor-declaration-control", false),
        ("imported-type", true),
        ("untyped", true),
        ("wrapped-mutation-control", false),
    ] {
        let path = root.join(format!("member-{name}.ts"));
        let source = std::fs::read_to_string(&path).unwrap();
        let embedded = extract_embedded_sql_from_source(&path, &source, &options);
        let [call] = embedded.calls.as_slice() else {
            panic!("{name}: one saved executor call");
        };
        let expected = if matches!(name, "untyped" | "imported-type") {
            2
        } else if stable {
            4
        } else if name == "distinct-control" {
            2
        } else {
            0
        };
        assert_eq!(call.bound_value_origins.len(), expected, "{name}");
        assert_eq!(super::collect(call, &path).len(), expected, "{name}");
    }
}
