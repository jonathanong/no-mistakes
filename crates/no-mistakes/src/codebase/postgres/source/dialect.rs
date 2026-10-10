//! PostgreSQL dialect that refuses `ON` as a select-item alias before `CONFLICT`.
//!
//! sqlparser consumes the alias keyword before asking the dialect, then restores
//! it when the dialect returns false. `ON` is not reserved for column aliases, so
//! `SELECT now() ON CONFLICT` was parsed as an alias named `ON`.
use sqlparser::dialect::{Dialect, PostgreSqlDialect, Precedence};
use sqlparser::keywords::Keyword;
use sqlparser::parser::{Parser, ParserError};
use std::any::TypeId;

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct PostgresSourceDialect;

impl Dialect for PostgresSourceDialect {
    fn dialect(&self) -> TypeId {
        // Preserve PostgreSQL parser branches that match on dialect TypeId.
        TypeId::of::<PostgreSqlDialect>()
    }

    fn is_select_item_alias(&self, explicit: bool, kw: &Keyword, parser: &mut Parser) -> bool {
        if !explicit && *kw == Keyword::ON && parser.peek_keyword(Keyword::CONFLICT) {
            return false;
        }
        PostgreSqlDialect {}.is_select_item_alias(explicit, kw, parser)
    }

    fn identifier_quote_style(&self, identifier: &str) -> Option<char> {
        PostgreSqlDialect {}.identifier_quote_style(identifier)
    }

    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        PostgreSqlDialect {}.is_delimited_identifier_start(ch)
    }

    fn is_identifier_start(&self, ch: char) -> bool {
        PostgreSqlDialect {}.is_identifier_start(ch)
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        PostgreSqlDialect {}.is_identifier_part(ch)
    }

    fn supports_unicode_string_literal(&self) -> bool {
        PostgreSqlDialect {}.supports_unicode_string_literal()
    }

    fn is_reserved_for_identifier(&self, kw: Keyword) -> bool {
        PostgreSqlDialect {}.is_reserved_for_identifier(kw)
    }

    fn is_table_alias(&self, kw: &Keyword, parser: &mut Parser) -> bool {
        PostgreSqlDialect {}.is_table_alias(kw, parser)
    }

    fn is_custom_operator_part(&self, ch: char) -> bool {
        PostgreSqlDialect {}.is_custom_operator_part(ch)
    }

    fn get_next_precedence(&self, parser: &Parser) -> Option<Result<u8, ParserError>> {
        PostgreSqlDialect {}.get_next_precedence(parser)
    }

    fn supports_filter_during_aggregation(&self) -> bool {
        PostgreSqlDialect {}.supports_filter_during_aggregation()
    }

    fn supports_group_by_expr(&self) -> bool {
        PostgreSqlDialect {}.supports_group_by_expr()
    }

    fn supports_alter_user_as_alter_role(&self) -> bool {
        PostgreSqlDialect {}.supports_alter_user_as_alter_role()
    }

    fn prec_value(&self, prec: Precedence) -> u8 {
        PostgreSqlDialect {}.prec_value(prec)
    }

    fn allow_extract_custom(&self) -> bool {
        PostgreSqlDialect {}.allow_extract_custom()
    }

    fn allow_extract_single_quotes(&self) -> bool {
        PostgreSqlDialect {}.allow_extract_single_quotes()
    }

    fn supports_create_index_with_clause(&self) -> bool {
        PostgreSqlDialect {}.supports_create_index_with_clause()
    }

    fn supports_explain_with_utility_options(&self) -> bool {
        PostgreSqlDialect {}.supports_explain_with_utility_options()
    }

    fn supports_listen_notify(&self) -> bool {
        PostgreSqlDialect {}.supports_listen_notify()
    }

    fn supports_exclude_constraint(&self) -> bool {
        PostgreSqlDialect {}.supports_exclude_constraint()
    }

    fn supports_factorial_operator(&self) -> bool {
        PostgreSqlDialect {}.supports_factorial_operator()
    }

    fn supports_bitwise_shift_operators(&self) -> bool {
        PostgreSqlDialect {}.supports_bitwise_shift_operators()
    }

    fn supports_comment_on(&self) -> bool {
        PostgreSqlDialect {}.supports_comment_on()
    }

    fn supports_load_extension(&self) -> bool {
        PostgreSqlDialect {}.supports_load_extension()
    }

    fn supports_named_fn_args_with_colon_operator(&self) -> bool {
        PostgreSqlDialect {}.supports_named_fn_args_with_colon_operator()
    }

    fn supports_named_fn_args_with_expr_name(&self) -> bool {
        PostgreSqlDialect {}.supports_named_fn_args_with_expr_name()
    }

    fn supports_empty_projections(&self) -> bool {
        PostgreSqlDialect {}.supports_empty_projections()
    }

    fn supports_nested_comments(&self) -> bool {
        PostgreSqlDialect {}.supports_nested_comments()
    }

    fn supports_string_escape_constant(&self) -> bool {
        PostgreSqlDialect {}.supports_string_escape_constant()
    }

    fn supports_numeric_literal_underscores(&self) -> bool {
        PostgreSqlDialect {}.supports_numeric_literal_underscores()
    }

    fn supports_array_typedef_with_brackets(&self) -> bool {
        PostgreSqlDialect {}.supports_array_typedef_with_brackets()
    }

    fn supports_geometric_types(&self) -> bool {
        PostgreSqlDialect {}.supports_geometric_types()
    }

    fn supports_order_by_using_operator(&self) -> bool {
        PostgreSqlDialect {}.supports_order_by_using_operator()
    }

    fn supports_set_names(&self) -> bool {
        PostgreSqlDialect {}.supports_set_names()
    }

    fn supports_alter_column_type_using(&self) -> bool {
        PostgreSqlDialect {}.supports_alter_column_type_using()
    }

    fn supports_left_associative_joins_without_parens(&self) -> bool {
        PostgreSqlDialect {}.supports_left_associative_joins_without_parens()
    }

    fn supports_notnull_operator(&self) -> bool {
        PostgreSqlDialect {}.supports_notnull_operator()
    }

    fn supports_interval_options(&self) -> bool {
        PostgreSqlDialect {}.supports_interval_options()
    }

    fn supports_insert_table_alias(&self) -> bool {
        PostgreSqlDialect {}.supports_insert_table_alias()
    }

    fn supports_create_table_like_parenthesized(&self) -> bool {
        PostgreSqlDialect {}.supports_create_table_like_parenthesized()
    }

    fn supports_select_wildcard_with_alias(&self) -> bool {
        PostgreSqlDialect {}.supports_select_wildcard_with_alias()
    }

    fn supports_comma_separated_trim(&self) -> bool {
        PostgreSqlDialect {}.supports_comma_separated_trim()
    }

    fn supports_xml_expressions(&self) -> bool {
        PostgreSqlDialect {}.supports_xml_expressions()
    }

    fn supports_aliased_function_args(&self) -> bool {
        PostgreSqlDialect {}.supports_aliased_function_args()
    }

    fn supports_comment_optimizer_hint(&self) -> bool {
        PostgreSqlDialect {}.supports_comment_optimizer_hint()
    }
}
