use super::*;

#[test]
fn masks_only_proven_siblings_and_preserves_original_ordinals() {
    let sql = "SELECT id FROM sql_placeholder_1 WHERE id < $1 sql_placeholder_2";
    assert_eq!(
        mask(sql, &[false, true]).as_deref(),
        Some("SELECT id FROM sql_placeholder_1 WHERE id < $1 /* sql_placeholder_2 */")
    );
    assert_eq!(mask(sql, &[true, false]).as_deref(), Some(sql));
    assert_eq!(mask("SELECT 1", &[]).as_deref(), Some("SELECT 1"));
}

#[test]
fn refuses_unaligned_or_ambiguous_markers() {
    for sql in [
        "SELECT sql_placeholder_",
        "SELECT sql_placeholder_0",
        "SELECT sql_placeholder_3",
        "SELECT sql_placeholder_99999999999999999999999999999",
        "SELECT sql_placeholder_1",
        "SELECT sql_placeholder_1, sql_placeholder_2, sql_placeholder_2",
    ] {
        assert_eq!(mask(sql, &[false, true]), None, "{sql}");
    }
}

#[test]
fn missing_sql_text_or_value_facts_never_create_identifier_findings() {
    use crate::codebase::check_facts::CheckFileFacts;
    use crate::codebase::rules::postgres_sql_cross_file::{compile, Options};
    use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
    let root = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
        .join("issue-3073-guard-composition");
    let file = root.join("conditional-fragment.ts");
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        root,
        Default::default(),
        Default::default(),
    );
    let opts = compile(RULE_ID, &Options::default()).unwrap();
    let embedded = crate::codebase::postgres::EmbeddedSqlCall::default();
    let mut call = SqlCall {
        file: file.clone(),
        span: (0, 0),
        call: &embedded,
    };
    let mut facts = CheckFactMap::default();
    assert!(sources(&graph, &opts, &facts, &call, &Bindings::new(&facts)).is_empty());
    let collected = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan {
            value_flow: true,
            identifier_provenance: true,
            ..TsFactPlan::imports()
        },
    );
    facts.files.push(file.clone());
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: std::sync::Arc::new(collected[&file].clone()),
            ..Default::default()
        }
        .into(),
    );
    let bindings = Bindings::new(&facts);
    assert!(sources(&graph, &opts, &facts, &call, &bindings).is_empty());
    for value in &facts.get_ts_facts(&file).unwrap().value_flow.calls {
        call.span = (value.offset, value.end);
        assert!(sources(&graph, &opts, &facts, &call, &bindings).is_empty());
    }
}

#[test]
fn required_projection_holes_and_glued_digits_keep_their_roles() {
    assert_eq!(mask("SELECT sql_placeholder_1, sql_placeholder_2 FROM sql_placeholder_3 WHERE id < $1 sql_placeholder_4", &[true, true, false, true]).as_deref(), Some("SELECT sql_placeholder_1, sql_placeholder_2 FROM sql_placeholder_3 WHERE id < $1 /* sql_placeholder_4 */"));
    assert_eq!(
        mask(
            "SELECT id FROM sql_placeholder_12 WHERE id < $1 sql_placeholder_2",
            &[true, true]
        )
        .as_deref(),
        Some("SELECT id FROM sql_placeholder_1_2 WHERE id < $1 /* sql_placeholder_2 */")
    );
    assert_eq!(
        mask(
            "SELECT sql_placeholder_1_suffix FROM fixed sql_placeholder_2",
            &[true, true]
        )
        .as_deref(),
        Some("SELECT sql_placeholder_1_suffix FROM fixed /* sql_placeholder_2 */")
    );
    assert_eq!(
        mask(
            "SELECT fixed_sql_placeholder_1 FROM fixed sql_placeholder_2",
            &[true, true]
        )
        .as_deref(),
        Some("SELECT fixed_sql_placeholder_1 FROM fixed /* sql_placeholder_2 */")
    );
}

#[test]
fn template_alias_recovery_rejects_mutability_cycles_and_missing_bindings() {
    use crate::codebase::check_facts::CheckFileFacts;
    use crate::codebase::dependencies::extract::ValueBinding;
    use crate::codebase::ts_source::facts::TsFileFacts;
    let file = std::path::PathBuf::from("query.ts");
    let named = |name: &str| ValueExpression::Named {
        name: name.into(),
        scope: Some(1),
        offset: 50,
        context: None,
    };
    let mut ts = TsFileFacts::default();
    for (index, (name, next)) in [
        ("first", "second"),
        ("second", "first"),
        ("mutable", "first"),
    ]
    .into_iter()
    .enumerate()
    {
        ts.value_flow.bindings.push(ValueBinding {
            immutable: name != "mutable",
            owner: None,
            name: name.into(),
            scope: 1,
            declared_at: index as u32 + 1,
            repeated: false,
            invalidated_at: None,
            mutation_contexts: Default::default(),
            value: named(next),
        });
    }
    let values = ["first", "mutable", "missing"].map(named);
    let mut facts = CheckFactMap::default();
    facts.files.push(file.clone());
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: std::sync::Arc::new(ts),
            ..Default::default()
        }
        .into(),
    );
    let bindings = Bindings::new(&facts);
    for value in &values {
        assert!(holes(
            &bindings,
            &file,
            value,
            None,
            None,
            &mut FxHashSet::default()
        )
        .is_none());
    }
}
