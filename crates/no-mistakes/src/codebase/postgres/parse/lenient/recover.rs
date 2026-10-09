use super::LocatedStatement;
mod bodies;
mod locations;
mod partition;
mod queries;
mod schema;
pub(super) use bodies::concatenated_strings;
use bodies::{peel_do_body, recover_chr_encoded};
use partition::recover_partition_change;
use schema::start as schema_ddl_start;
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn parse_chunks_with_function_calls(
    chunks: Vec<Vec<Token>>,
    original: &[TokenWithSpan],
    allow_concurrent_detach: bool,
) -> Vec<LocatedStatement> {
    let mut located = original
        .split(|token| token.token == Token::SemiColon)
        .filter(|chunk| {
            chunk
                .iter()
                .any(|token| !matches!(token.token, Token::Whitespace(_)))
        });
    let mut statements = Vec::new();
    let mut transaction_open = false;
    for chunk in chunks {
        let source = located
            .next()
            .map(|source| locations::align(&chunk, source));
        for statement in parse_chunk(
            chunk,
            source.as_deref(),
            allow_concurrent_detach && !transaction_open,
        ) {
            match &statement.statement {
                Statement::StartTransaction { .. } => transaction_open = true,
                Statement::Commit { chain, .. }
                | Statement::Rollback {
                    savepoint: None,
                    chain,
                } => {
                    transaction_open = *chain;
                }
                _ => {}
            }
            statements.push(statement);
        }
    }
    statements
}

fn parse_chunk(
    chunk: Vec<Token>,
    original: Option<&[TokenWithSpan]>,
    allow_concurrent_detach: bool,
) -> Vec<LocatedStatement> {
    if let Some(body) = peel_do_body(&chunk) {
        let body = locations::align_do_body(&body, original);
        return super::parse_with_sources(&body, true, false)
            .into_iter()
            .filter(|located| !is_begin_or_end(&located.statement))
            .collect();
    }
    let dialect = PostgreSqlDialect {};
    let mut parser = match original {
        Some(tokens) => Parser::new(&dialect).with_tokens_with_locations(tokens.to_vec()),
        None => Parser::new(&dialect).with_tokens(chunk.clone()),
    };
    match parser.parse_statement() {
        Ok(statement) if matches!(parser.peek_token().token, Token::EOF) => {
            vec![LocatedStatement::plain(statement)]
        }
        _ => {
            if let Some(tokens) = original {
                if let Some((statement, functions)) =
                    crate::codebase::postgres::source::recover_conflict_calls(tokens)
                {
                    let mut located = LocatedStatement::plain(statement);
                    located.recovered_functions = functions;
                    return vec![located];
                }
            }
            if let Some(partition_change) =
                recover_partition_change(&chunk, original, allow_concurrent_detach)
            {
                return vec![LocatedStatement::plain(partition_change)];
            }
            let recovered = recover_chr_encoded(&chunk, original, allow_concurrent_detach);
            if recovered.is_empty() {
                let expressions = queries::recover(&chunk, original);
                // Keep the established guarded DML/DDL recovery alongside any
                // newly recovered condition expression, without replacing it.
                let mut projected: Vec<_> = expressions
                    .into_iter()
                    .map(|(statement, synthetic)| LocatedStatement::functions(statement, synthetic))
                    .collect();
                projected.extend(
                    recover_schema_ddl(&chunk, original, allow_concurrent_detach)
                        .map(LocatedStatement::plain),
                );
                projected
            } else {
                recovered
            }
        }
    }
}

fn is_begin_or_end(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::StartTransaction { .. } | Statement::Commit { .. }
    )
}

fn recover_schema_ddl(
    tokens: &[Token],
    original: Option<&[TokenWithSpan]>,
    allow_concurrent_detach: bool,
) -> Option<Statement> {
    let start = schema_ddl_start(tokens)?;
    // PL/pgSQL wrappers can precede a complete partition transition. Apply the
    // same narrow recovery to the DDL suffix before using the PostgreSQL parser.
    if let Some(change) = recover_partition_change(
        &tokens[start..],
        original.map(|tokens| &tokens[start..]),
        allow_concurrent_detach,
    ) {
        return Some(change);
    }
    let dialect = PostgreSqlDialect {};
    let mut parser = match original {
        Some(tokens) => Parser::new(&dialect).with_tokens_with_locations(tokens[start..].to_vec()),
        None => Parser::new(&dialect).with_tokens(tokens[start..].to_vec()),
    };
    parser.parse_statement().ok()
}

#[cfg(test)]
mod tests;
