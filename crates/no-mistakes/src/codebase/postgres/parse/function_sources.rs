use super::LocatedStatement;
use crate::codebase::postgres::SqlFunctionCallFact;
use sqlparser::tokenizer::TokenWithSpan;
use std::collections::HashSet;
use std::sync::Arc;

/// Chunks whose tokens never appear on a kept statement. A failed chunk that
/// recovered no INSERT still has to abstain on its own `ON CONFLICT`.
pub(crate) fn orphan_chunk_tokens(statements: &[LocatedStatement]) -> Vec<Arc<[TokenWithSpan]>> {
    let kept = statements
        .iter()
        .filter(|statement| !statement.function_projection)
        .filter_map(|statement| {
            statement
                .chunk_tokens
                .as_ref()
                .map(|tokens| Arc::as_ptr(tokens) as *const () as usize)
        })
        .collect::<HashSet<_>>();
    let mut seen = HashSet::new();
    let mut orphans = Vec::new();
    for statement in statements {
        let Some(tokens) = &statement.chunk_tokens else {
            continue;
        };
        let ptr = Arc::as_ptr(tokens) as *const () as usize;
        if kept.contains(&ptr) || !seen.insert(ptr) {
            continue;
        }
        orphans.push(Arc::clone(tokens));
    }
    orphans
}

/// Partition one shared parse into the established AST and auxiliary syntactic
/// call facts; procedural expressions never enter row-bound/lifecycle inputs.
pub(crate) fn partition(
    located: Vec<LocatedStatement>,
) -> (Vec<LocatedStatement>, Vec<SqlFunctionCallFact>) {
    let mut statements = Vec::new();
    let mut functions = Vec::new();
    for mut statement in located {
        functions.append(&mut statement.recovered_functions);
        if statement.function_projection {
            crate::codebase::postgres::function_calls::collect_projected(
                &statement.statement,
                &mut functions,
                statement.synthetic_select,
            );
        } else {
            statements.push(statement);
        }
    }
    (statements, functions)
}
