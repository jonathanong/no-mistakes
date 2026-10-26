use super::LocatedStatement;
mod bodies;
mod copy;
pub(crate) mod failure;
pub(super) use failure::failed_source;
mod locations;
mod partition;
mod procedural;
mod queries;
mod query_locks;
mod schema;
mod select_into;
pub(super) use bodies::concatenated_strings;
use bodies::{peel_do_body, recover_chr_encoded};
use partition::recover_partition_change;
pub(crate) use procedural::BodyState;
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan};
use std::sync::Arc;

pub(super) fn parse_chunks_with_function_calls(
    chunks: Vec<Vec<Token>>,
    original: &[TokenWithSpan],
    allow_concurrent_detach: bool,
    procedural_body: bool,
    recursive_views: &super::super::RecursiveViews,
) -> Vec<LocatedStatement> {
    let mut located = super::chunks::located_chunks(original);
    let mut statements = Vec::new();
    let mut transaction_open = false;
    let mut copy_data_pending = false;
    let mut ordinal = 0;
    for chunk in chunks {
        let original = located.next();
        if copy_data_pending && original.is_some_and(copy::is_terminator) {
            copy_data_pending = false;
            continue;
        }
        let chunk_ordinal = ordinal;
        ordinal += 1;
        let aligned = original.map(|source| locations::align(&chunk, source));
        // Keep each chunk's tokens with its own failure marker.
        let tokens = original.map(Arc::<[TokenWithSpan]>::from);
        let mut handled_procedural = None;
        let mut produced = parse_chunk(
            chunk,
            aligned.as_deref(),
            allow_concurrent_detach && !transaction_open,
            &mut handled_procedural,
            procedural_body,
        );
        failure::reject_incompatible(&mut produced, original, procedural_body, recursive_views);
        if handled_procedural.is_none() {
            failure::partial_chunk(original, &mut produced);
        }
        copy_data_pending = original.is_some_and(copy::owns_terminator);
        if produced.is_empty() || handled_procedural == Some(false) {
            if handled_procedural == Some(true) {
                continue;
            }
            if let Some(tokens) = &tokens {
                statements.push(failure::marker(
                    tokens,
                    chunk_ordinal,
                    handled_procedural == Some(false),
                    &produced,
                ));
            }
            if produced.is_empty() {
                continue;
            }
        }
        for mut statement in produced {
            if let Some(failure) = &mut statement.recovery_failure {
                failure.source_order.insert(0, chunk_ordinal);
            }
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
            statement.chunk_tokens = tokens.clone();
            statements.push(statement);
        }
    }
    statements
}
fn parse_chunk(
    chunk: Vec<Token>,
    original: Option<&[TokenWithSpan]>,
    allow_concurrent_detach: bool,
    handled_procedural: &mut Option<bool>,
    procedural_body: bool,
) -> Vec<LocatedStatement> {
    if let Some(body) = peel_do_body(&chunk) {
        let body = locations::align_do_body(&body, original);
        // Keep recovered facts alongside the outer marker if body lexing fails.
        *handled_procedural = Some(
            !body.trim().is_empty()
                && !super::super::unicode::tokenize_with_location(&body, false).is_empty(),
        );
        return procedural::filter_body(
            super::parse_with_sources(&body, true, false, true),
            *handled_procedural == Some(false),
        );
    }
    let dialect = PostgreSqlDialect {};
    let mut parse_tokens = original.map_or_else(
        || chunk.iter().cloned().map(TokenWithSpan::wrap).collect(),
        |tokens| tokens.to_vec(),
    );
    super::query_compat::rewrite(&mut parse_tokens);
    let create_valid = super::create_compat::valid_original(original);
    super::create_compat::rewrite(&mut parse_tokens);
    let mut targets_valid = create_valid;
    if chunk
        .iter()
        .find(|token| !matches!(token, Token::Whitespace(_)))
        .is_some_and(|token| {
            matches!(token, Token::Word(word)
            if word.quote_style.is_none() && (word.value.eq_ignore_ascii_case("SELECT")
                || word.value.eq_ignore_ascii_case("WITH")))
        })
    {
        // Retain a complete query AST, including INTO and its original spans.
        query_locks::rewrite_located(&mut parse_tokens);
        if procedural_body {
            targets_valid = select_into::normalize(&mut parse_tokens);
        }
    }
    let mut parser = Parser::new(&dialect).with_tokens_with_locations(parse_tokens);
    match parser.parse_statement() {
        Ok(statement) if targets_valid && matches!(parser.peek_token().token, Token::EOF) => {
            let mut located = LocatedStatement::plain(statement);
            located.complete = true;
            vec![located]
        }
        _ if !create_valid => Vec::new(),
        _ => {
            if let Some(tokens) = original {
                if let Some((statement, functions, arbiter)) =
                    crate::codebase::postgres::source::recover_conflict_calls(tokens)
                {
                    let mut located = LocatedStatement::plain(statement);
                    located.recovered_functions = functions;
                    located.policy_arbiter = arbiter;
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
                let expressions = queries::recover(&chunk, original, procedural_body);
                let action_projected = expressions.iter().any(|(_, synthetic)| !synthetic);
                // Preserve guarded DML/DDL recovery alongside the condition.
                let mut projected: Vec<_> = expressions
                    .into_iter()
                    .map(|(statement, synthetic)| LocatedStatement::functions(statement, synthetic))
                    .collect();
                let schema = schema::recover(&chunk, original, allow_concurrent_detach);
                let has_schema = schema.is_some();
                if schema.as_ref().is_some_and(|item| item.complete) {
                    // The complete utility also validated its IF/ELSIF guard;
                    // its synthetic condition is not partial statement recovery.
                    for item in &mut projected {
                        item.complete = true;
                    }
                }
                projected.extend(schema);
                if !has_schema && !action_projected {
                    projected.extend(failure::guarded_action(&chunk, original));
                }
                projected
            } else {
                recovered
            }
        }
    }
}
#[cfg(test)]
mod tests;
