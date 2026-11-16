//! What the walk records for calls built from imported helpers.

use super::*;
use crate::codebase::postgres::embedded::helpers::PendingImportedSql;
use crate::codebase::postgres::EmbeddedSqlCall;

fn pending_for(facts: &EmbeddedSqlFileFacts, index: usize) -> Option<&PendingImportedSql> {
    let span = facts.call_spans[index];
    facts
        .pending_imported
        .iter()
        .find(|pending| pending.span == span)
}

fn call(facts: &EmbeddedSqlFileFacts, index: usize) -> &EmbeddedSqlCall {
    &facts.calls[index]
}

#[path = "pending_cases/part1.rs"]
mod part1;

#[path = "pending_cases/part2.rs"]
mod part2;
