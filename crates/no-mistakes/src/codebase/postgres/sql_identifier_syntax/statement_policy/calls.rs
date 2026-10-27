use super::super::{helper_recovery::Recovered, SqlCall};
use crate::codebase::postgres::{
    EmbeddedSqlCall, EmbeddedSqlFileFacts, EmbeddedSqlKind, EmbeddedSqlVariant,
};
use std::path::Path;

pub(super) fn empty(path: &Path) -> EmbeddedSqlFileFacts {
    EmbeddedSqlFileFacts {
        path: path.to_path_buf(),
        executor_bindings: Vec::new(),
        calls: Vec::new(),
        undeclared_executor_types: Vec::new(),
        call_spans: Vec::new(),
        exported_helpers: Default::default(),
        exported_programs: Default::default(),
        exported_constants: Default::default(),
        pending_imported: Vec::new(),
        imported_call_lines: Default::default(),
        source_comment_spans: Vec::new(),
        fragments: Vec::new(),
        fragment_sites: Vec::new(),
        fragment_variants: Vec::new(),
        matched_factory_names: Vec::new(),
        matched_type_names: Vec::new(),
        pending_relative: Default::default(),
        unproven_member_query_spans: Vec::new(),
        postgres_rule_ids: Vec::new(),
        record_unproven_spans: false,
    }
}
pub(super) fn unknown(call: &SqlCall) -> EmbeddedSqlCall {
    EmbeddedSqlCall {
        line: call.line,
        kind: EmbeddedSqlKind::Dynamic,
        ..Default::default()
    }
}
pub(super) fn recovered(call: &SqlCall, proof: &Recovered) -> EmbeddedSqlCall {
    if proof.nonexecuting && proof.complete() {
        // A proven nonexecution has no obligations, represented by an empty valid SQL batch.
        return EmbeddedSqlCall {
            line: call.line,
            sql_text: Some(String::new()),
            ..Default::default()
        };
    }
    EmbeddedSqlCall {
        line: call.line,
        kind: if proof.complete() {
            EmbeddedSqlKind::Composed
        } else {
            EmbeddedSqlKind::Dynamic
        },
        variants: proof
            .alternatives
            .iter()
            .map(|sql| EmbeddedSqlVariant {
                sql_text: sql.sql.clone(),
                sql_source_offsets: sql.origins.clone(),
                line: sql.line,
                sql_source_positions: sql.positions.clone(),
                recovered_placeholder_positions: Vec::new(),
                append_sites: Vec::new(),
                variants_truncated: !proof.complete(),
            })
            .collect(),
        ..Default::default()
    }
}

/// Only the recovered argument's root helper owns this SQL call; sibling calls do not.
pub(super) fn helper_lines(
    file: &crate::codebase::check_facts::CheckFileFacts,
    call: &SqlCall,
) -> Vec<u32> {
    use crate::codebase::postgres::embedded::helpers::program::{Expr, Stmt};
    let models = &file.sql_identifier_syntax.recovery.calls;
    let Some(argument) = models
        .binary_search_by_key(&call.span, |call| call.span)
        .ok()
        .and_then(|index| models[index].arguments.first())
    else {
        return Vec::new();
    };
    match argument.program.body.as_slice() {
        [Stmt::Return(Some(Expr::Call(_, _)), mark)] => vec![mark.line],
        _ => Vec::new(),
    }
}
