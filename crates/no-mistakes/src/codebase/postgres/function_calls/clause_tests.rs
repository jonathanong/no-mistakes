use super::*;
use std::collections::BTreeMap;

#[test]
fn clause_names_and_labels_preserve_the_configuration_contract() {
    let cases = [
        (SqlFunctionClause::Where, "where", "WHERE"),
        (SqlFunctionClause::JoinOn, "join-on", "JOIN ON"),
        (SqlFunctionClause::Having, "having", "HAVING"),
        (SqlFunctionClause::SelectList, "select-list", "SELECT list"),
        (SqlFunctionClause::OrderBy, "order-by", "ORDER BY"),
        (SqlFunctionClause::Values, "values", "VALUES"),
        (SqlFunctionClause::Set, "set", "SET"),
        (SqlFunctionClause::Default, "default", "DEFAULT"),
        (SqlFunctionClause::Returning, "returning", "RETURNING"),
    ];
    for (clause, name, label) in cases {
        assert_eq!(SqlFunctionClause::parse(name), Some(clause));
        assert_eq!(clause.as_str(), name);
        assert_eq!(clause.label(), label);
    }
    for invalid in ["", "WHERE", "where ", "from", "group-by"] {
        assert_eq!(SqlFunctionClause::parse(invalid), None);
    }
}

#[test]
fn nested_queries_and_function_clauses_restore_their_enclosing_context() {
    use SqlFunctionClause::*;
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres/function-call-clauses/queries.sql"
    ));
    let statements = crate::codebase::postgres::parse::parse_postgres_sql(source).unwrap();
    let mut calls = Vec::new();
    for statement in &statements {
        collect(statement, &mut calls);
    }
    let actual = calls
        .iter()
        .filter(|call| call.name_parts[0].starts_with("probe_"))
        .map(|call| (call.name_parts[0].as_str(), call.clause))
        .collect::<BTreeMap<_, _>>();
    let expected = BTreeMap::from([
        ("probe_select_list", Some(SelectList)),
        ("probe_select_arg", Some(SelectList)),
        ("probe_where", Some(Where)),
        ("probe_nested_select", Some(SelectList)),
        ("probe_nested_where", Some(Where)),
        ("probe_restored_where", Some(Where)),
        ("probe_unscoped_group", None),
        ("probe_having", Some(Having)),
        ("probe_order_by", Some(OrderBy)),
        ("probe_from", None),
        ("probe_from_arg", None),
        ("probe_join_on", Some(JoinOn)),
        ("probe_values_left", Some(Values)),
        ("probe_values_right", Some(Values)),
        ("probe_argument_order", Some(OrderBy)),
        ("probe_filter_where", Some(Where)),
        ("probe_inline_where", Some(Where)),
        ("probe_within_group_order", Some(OrderBy)),
        ("probe_window_order", Some(OrderBy)),
        ("probe_named_window_order", Some(OrderBy)),
        ("probe_nested_projection", Some(SelectList)),
        ("probe_after_subquery", Some(Where)),
    ]);
    assert_eq!(actual, expected);
    let mut again = Vec::new();
    for statement in &statements {
        collect(statement, &mut again);
    }
    assert_eq!(again, calls);
}

#[test]
fn unsupported_window_and_synthetic_routine_roots_remain_unscoped() {
    use SqlFunctionClause::*;
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres/function-call-clauses/boundaries.sql"
    ));
    let parsed = crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(source);
    let (statements, mut calls) =
        crate::codebase::postgres::parse::partition_function_sources(parsed);
    for statement in &statements {
        collect(&statement.statement, &mut calls);
    }
    let actual = calls
        .iter()
        .filter(|call| call.name_parts[0].starts_with("probe_"))
        .map(|call| (call.name_parts[0].as_str(), call.clause))
        .collect::<BTreeMap<_, _>>();
    let expected = BTreeMap::from([
        ("probe_inline_partition", None),
        ("probe_named_partition", None),
        ("probe_inline_start", None),
        ("probe_inline_end", None),
        ("probe_named_start", None),
        ("probe_named_end", None),
        ("probe_inline_order", Some(OrderBy)),
        ("probe_named_order", Some(OrderBy)),
        ("probe_if", None),
        ("probe_perform", None),
        ("probe_return", None),
        ("probe_guard", None),
        ("probe_real_select", Some(SelectList)),
        ("probe_real_where", Some(Where)),
        ("probe_return_query", Some(SelectList)),
        ("probe_nested_real_select", Some(SelectList)),
        ("probe_nested_real_where", Some(Where)),
    ]);
    assert_eq!(actual, expected);
}
