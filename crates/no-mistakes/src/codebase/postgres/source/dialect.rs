//! PostgreSQL dialect that refuses `ON` as a select-item alias before `CONFLICT`.
//!
//! sqlparser consumes the alias keyword before asking the dialect, then restores
//! it when the dialect returns false. `ON` is not reserved for column aliases, so
//! `SELECT now() ON CONFLICT` was parsed as an alias named `ON`.
use sqlparser::dialect::{Dialect, PostgreSqlDialect, Precedence};
use sqlparser::keywords::Keyword;
use sqlparser::parser::{Parser, ParserError};
use sqlparser::tokenizer::Token;
use std::any::TypeId;

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct PostgresSourceDialect;

macro_rules! pg_bool {
    ($($name:ident),* $(,)?) => {
        $(
            fn $name(&self) -> bool {
                PostgreSqlDialect {}.$name()
            }
        )*
    };
}

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
        // sqlparser 0.64 assigns `:=` unknown precedence so it is not an infix
        // operator. Parenthesized assignments still need that infix form.
        if parser.peek_token_ref().token == Token::Assignment {
            return Some(Ok(self.prec_value(Precedence::PgOther)));
        }
        PostgreSqlDialect {}.get_next_precedence(parser)
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

    pg_bool!(
        supports_array_typedef_with_brackets,
        supports_aliased_function_args,
        supports_alter_column_type_using,
        supports_alter_user_as_alter_role,
        supports_bitwise_shift_operators,
        supports_comma_separated_trim,
        supports_comment_on,
        supports_comment_optimizer_hint,
        supports_comparison_in_between_low_bound,
        supports_create_group,
        supports_create_index_with_clause,
        supports_create_table_like_parenthesized,
        supports_cte_materialization,
        supports_drop_group,
        supports_exclude_constraint,
        supports_explain_with_utility_options,
        supports_factorial_operator,
        supports_filter_during_aggregation,
        supports_geometric_types,
        supports_group_by_expr,
        supports_insert_table_alias,
        supports_interval_options,
        supports_left_associative_joins_without_parens,
        supports_listen_notify,
        supports_load_extension,
        supports_named_fn_args_with_assignment_operator,
        supports_named_fn_args_with_colon_operator,
        supports_named_fn_args_with_expr_name,
        supports_nested_comments,
        supports_non_decimal_integer_literals,
        supports_notnull_operator,
        supports_numeric_literal_underscores,
        supports_order_by_using_operator,
        supports_select_wildcard_with_alias,
        supports_set_names,
        supports_sql_option_without_value,
        supports_string_escape_constant,
        supports_tilde_like_match_operator,
        supports_unicode_string_literal,
        supports_xml_expressions,
    );
}
