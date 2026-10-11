use super::{PreparedSql, StatementPolicySources};
use crate::codebase::postgres::{SqlFunctionCallFact, SqlSettingUse, SqlStatementKind};
use sqlparser::ast::Statement;

pub(super) fn collect(
    sql: &str,
    prepared_sql: &PreparedSql<'_>,
    statements: &[Statement],
    policy: StatementPolicySources<'_>,
) -> (
    Vec<SqlStatementKind>,
    Vec<SqlSettingUse>,
    Vec<SqlFunctionCallFact>,
) {
    let (statement_kinds, setting_uses, mut function_calls) = policy.schema.map_or_else(
        || {
            let facts = crate::codebase::postgres::migration::policy_facts(sql, statements);
            (
                facts.statement_kinds,
                facts.setting_uses,
                facts.function_calls,
            )
        },
        |facts| {
            (
                facts.statement_kinds.clone(),
                facts.setting_uses.clone(),
                facts.function_calls.clone(),
            )
        },
    );
    if policy.schema.is_none() {
        function_calls.extend_from_slice(policy.functions);
        function_calls.extend_from_slice(&prepared_sql.functions());
    }
    (statement_kinds, setting_uses, function_calls)
}

pub(super) fn table_keyword(token: &sqlparser::tokenizer::TokenWithSpan) -> bool {
    matches!(&token.token, sqlparser::tokenizer::Token::Word(word) if word.keyword == sqlparser::keywords::Keyword::TABLE)
}
