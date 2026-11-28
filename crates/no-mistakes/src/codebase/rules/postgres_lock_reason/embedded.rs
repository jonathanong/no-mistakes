mod variant;

use super::comments_index::CommentIndex;
use super::findings::{push_undeclared_executors, unanalyzable_finding, FileOutcome};
use super::options::CompiledOptions;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{EmbeddedSqlCall, EmbeddedSqlKind};
use crate::codebase::ts_source::SourceStore;
use anyhow::Result;
use std::path::Path;

pub(super) struct EmbeddedCtx<'a> {
    pub(super) options: &'a CompiledOptions,
    pub(super) path: &'a Path,
    pub(super) rel: &'a str,
    pub(super) sources: &'a SourceStore,
    pub(super) ts_comments: &'a mut Option<CommentIndex>,
    pub(super) out: &'a mut FileOutcome,
}

pub(super) fn collect(
    options: &CompiledOptions,
    path: &Path,
    rel: &str,
    sources: &SourceStore,
    facts: &dyn SqlFactContext,
    ts_comments: &mut Option<CommentIndex>,
    out: &mut FileOutcome,
) -> Result<()> {
    let embedded = facts.embedded_sql(path, &options.embedded)?;
    let mut statements = facts.postgres_statements(path, Some(&options.embedded))?;
    push_undeclared_executors(
        rel,
        &embedded.undeclared_executor_types,
        &mut out.unanalyzable,
    );
    let mut ctx = EmbeddedCtx {
        options,
        path,
        rel,
        sources,
        ts_comments,
        out,
    };
    for call in &embedded.calls {
        let (versions, remaining) = statements.split_at(variant_count(call));
        statements = remaining;
        if is_unresolved_call(call, versions.is_empty()) {
            ctx.out.unanalyzable.push(unanalyzable_finding(
                ctx.rel,
                call.line as usize,
                "executor SQL could not be resolved",
            ));
            continue;
        }
        for (variant_index, statement) in versions.iter().enumerate() {
            variant::check_variant(&mut ctx, call, variant_index, statement);
        }
    }
    debug_assert!(statements.is_empty());
    Ok(())
}

fn variant_count(call: &EmbeddedSqlCall) -> usize {
    if call.variants.is_empty() {
        usize::from(call.sql_text.is_some())
    } else {
        call.variants.len()
    }
}

fn is_unresolved_call(call: &EmbeddedSqlCall, versions_empty: bool) -> bool {
    call.variants.is_empty() && (versions_empty || call.kind == EmbeddedSqlKind::Dynamic)
}
