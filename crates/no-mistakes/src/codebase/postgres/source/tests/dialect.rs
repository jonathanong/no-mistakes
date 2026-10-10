use super::super::dialect::PostgresSourceDialect;
use sqlparser::dialect::{Dialect, PostgreSqlDialect, Precedence};
use sqlparser::keywords::Keyword;
use sqlparser::parser::Parser;

#[test]
fn source_dialect_matches_postgresql_except_on_conflict_alias() {
    let source = PostgresSourceDialect;
    let postgres = PostgreSqlDialect {};
    assert_eq!(source.dialect(), postgres.dialect());
    assert_eq!(
        source.identifier_quote_style("name"),
        postgres.identifier_quote_style("name")
    );
    assert_eq!(
        source.is_delimited_identifier_start('"'),
        postgres.is_delimited_identifier_start('"')
    );
    assert_eq!(
        source.is_identifier_start('雪'),
        postgres.is_identifier_start('雪')
    );
    assert_eq!(
        source.is_identifier_part('$'),
        postgres.is_identifier_part('$')
    );
    assert_eq!(
        source.supports_unicode_string_literal(),
        postgres.supports_unicode_string_literal()
    );
    assert_eq!(
        source.is_reserved_for_identifier(Keyword::SELECT),
        postgres.is_reserved_for_identifier(Keyword::SELECT)
    );
    assert_eq!(
        source.is_custom_operator_part('+'),
        postgres.is_custom_operator_part('+')
    );
    assert_eq!(
        source.supports_filter_during_aggregation(),
        postgres.supports_filter_during_aggregation()
    );
    assert_eq!(
        source.supports_group_by_expr(),
        postgres.supports_group_by_expr()
    );
    assert_eq!(
        source.supports_alter_user_as_alter_role(),
        postgres.supports_alter_user_as_alter_role()
    );
    assert_eq!(
        source.prec_value(Precedence::PlusMinus),
        postgres.prec_value(Precedence::PlusMinus)
    );
    assert_eq!(
        source.allow_extract_custom(),
        postgres.allow_extract_custom()
    );
    assert_eq!(
        source.allow_extract_single_quotes(),
        postgres.allow_extract_single_quotes()
    );
    assert_eq!(
        source.supports_create_index_with_clause(),
        postgres.supports_create_index_with_clause()
    );
    assert_eq!(
        source.supports_explain_with_utility_options(),
        postgres.supports_explain_with_utility_options()
    );
    assert_eq!(
        source.supports_listen_notify(),
        postgres.supports_listen_notify()
    );
    assert_eq!(
        source.supports_exclude_constraint(),
        postgres.supports_exclude_constraint()
    );
    assert_eq!(
        source.supports_factorial_operator(),
        postgres.supports_factorial_operator()
    );
    assert_eq!(
        source.supports_bitwise_shift_operators(),
        postgres.supports_bitwise_shift_operators()
    );
    assert_eq!(source.supports_comment_on(), postgres.supports_comment_on());
    assert_eq!(
        source.supports_load_extension(),
        postgres.supports_load_extension()
    );
    assert_eq!(
        source.supports_named_fn_args_with_colon_operator(),
        postgres.supports_named_fn_args_with_colon_operator()
    );
    assert_eq!(
        source.supports_named_fn_args_with_expr_name(),
        postgres.supports_named_fn_args_with_expr_name()
    );
    assert_eq!(
        source.supports_empty_projections(),
        postgres.supports_empty_projections()
    );
    assert_eq!(
        source.supports_nested_comments(),
        postgres.supports_nested_comments()
    );
    assert_eq!(
        source.supports_string_escape_constant(),
        postgres.supports_string_escape_constant()
    );
    assert_eq!(
        source.supports_numeric_literal_underscores(),
        postgres.supports_numeric_literal_underscores()
    );
    assert_eq!(
        source.supports_array_typedef_with_brackets(),
        postgres.supports_array_typedef_with_brackets()
    );
    assert_eq!(
        source.supports_geometric_types(),
        postgres.supports_geometric_types()
    );
    assert_eq!(
        source.supports_order_by_using_operator(),
        postgres.supports_order_by_using_operator()
    );
    assert_eq!(source.supports_set_names(), postgres.supports_set_names());
    assert_eq!(
        source.supports_alter_column_type_using(),
        postgres.supports_alter_column_type_using()
    );
    assert_eq!(
        source.supports_left_associative_joins_without_parens(),
        postgres.supports_left_associative_joins_without_parens()
    );
    assert_eq!(
        source.supports_notnull_operator(),
        postgres.supports_notnull_operator()
    );
    assert_eq!(
        source.supports_interval_options(),
        postgres.supports_interval_options()
    );
    assert_eq!(
        source.supports_insert_table_alias(),
        postgres.supports_insert_table_alias()
    );
    assert_eq!(
        source.supports_create_table_like_parenthesized(),
        postgres.supports_create_table_like_parenthesized()
    );
    assert_eq!(
        source.supports_select_wildcard_with_alias(),
        postgres.supports_select_wildcard_with_alias()
    );
    assert_eq!(
        source.supports_comma_separated_trim(),
        postgres.supports_comma_separated_trim()
    );
    assert_eq!(
        source.supports_xml_expressions(),
        postgres.supports_xml_expressions()
    );
    assert_eq!(
        source.supports_aliased_function_args(),
        postgres.supports_aliased_function_args()
    );
    assert_eq!(
        source.supports_comment_optimizer_hint(),
        postgres.supports_comment_optimizer_hint()
    );

    let mut source_alias = Parser::new(&source).try_with_sql("SELECT 1").unwrap();
    let mut postgres_alias = Parser::new(&postgres).try_with_sql("SELECT 1").unwrap();
    assert_eq!(
        source.is_table_alias(&Keyword::AS, &mut source_alias),
        postgres.is_table_alias(&Keyword::AS, &mut postgres_alias)
    );
    let source_expr = Parser::new(&source).try_with_sql("a + 1").unwrap();
    let postgres_expr = Parser::new(&postgres).try_with_sql("a + 1").unwrap();
    assert_eq!(
        format!("{:?}", source.get_next_precedence(&source_expr)),
        format!("{:?}", postgres.get_next_precedence(&postgres_expr))
    );

    let mut on_conflict = Parser::new(&source).try_with_sql("CONFLICT").unwrap();
    assert!(!source.is_select_item_alias(false, &Keyword::ON, &mut on_conflict));
    let mut explicit = Parser::new(&source).try_with_sql("CONFLICT").unwrap();
    let mut explicit_postgres = Parser::new(&postgres).try_with_sql("CONFLICT").unwrap();
    assert_eq!(
        source.is_select_item_alias(true, &Keyword::ON, &mut explicit),
        postgres.is_select_item_alias(true, &Keyword::ON, &mut explicit_postgres)
    );
}
