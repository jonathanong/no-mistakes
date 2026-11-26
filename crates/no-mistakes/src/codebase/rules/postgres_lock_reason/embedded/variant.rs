use super::EmbeddedCtx;
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::{EmbeddedSqlCall, SqlStatementFileFacts};
use crate::codebase::rules::postgres_lock_inventory::embedded::Positions;
use crate::codebase::rules::postgres_lock_reason::comment::ReasonStatus;
use crate::codebase::rules::postgres_lock_reason::comments_index::CommentIndex;
use crate::codebase::rules::postgres_lock_reason::findings::{
    is_lock_category, load_ts_comments, matches_sql_helper, push_reason_finding,
    statement_has_lock_work, unanalyzable_finding, PARSE_GAP, ROUTINE_GAP,
};
use crate::codebase::rules::postgres_lock_reason::options::CompiledOptions;
use crate::codebase::rules::RuleFinding;

struct VariantContext<'a> {
    rel: &'a str,
    call_line: usize,
    sql_comments: &'a CommentIndex,
    ts_index: Option<&'a CommentIndex>,
    call_status: ReasonStatus,
}

pub(super) fn check_variant(
    ctx: &mut EmbeddedCtx<'_>,
    call: &EmbeddedSqlCall,
    variant_index: usize,
    statement: &SqlStatementFileFacts,
) {
    if is_truncated_variant(call, variant_index) {
        ctx.out.unanalyzable.push(unanalyzable_finding(
            ctx.rel,
            call.line as usize,
            "executor SQL alternatives are incomplete",
        ));
        return;
    }
    let mut positions = None;
    record_variant_gaps(
        ctx.rel,
        call,
        variant_index,
        statement,
        &mut positions,
        &mut ctx.out.unanalyzable,
    );
    if statement.parse_failed || !statement_has_lock_work(ctx.options, statement) {
        return;
    }
    let sql_comments = CommentIndex::for_sql(variant_sql_text(call, variant_index));
    let ts_index = load_ts_comments(ctx.path, ctx.sources, ctx.ts_comments);
    let call_status = ts_index.map_or(ReasonStatus::Absent, |comments| {
        comments.status_for_ts_line(call.line as usize)
    });
    let positions = positions.get_or_insert_with(|| Positions::new(call, variant_index));
    let check = VariantContext {
        rel: ctx.rel,
        call_line: call.line as usize,
        sql_comments: &sql_comments,
        ts_index,
        call_status,
    };
    check_embedded_locks(&check, statement, positions, &mut ctx.out.findings);
    check_embedded_helpers(
        ctx.options,
        &check,
        statement,
        positions,
        &mut ctx.out.findings,
    );
}

fn is_truncated_variant(call: &EmbeddedSqlCall, variant_index: usize) -> bool {
    call.variants
        .get(variant_index)
        .is_some_and(|variant| variant.variants_truncated)
}

fn record_variant_gaps<'c>(
    rel: &str,
    call: &'c EmbeddedSqlCall,
    variant_index: usize,
    statement: &SqlStatementFileFacts,
    positions: &mut Option<Positions<'c>>,
    unanalyzable: &mut Vec<RuleFinding>,
) {
    for (site, line) in statement.parse_failure_sites() {
        let report_line = if site_is_imported(statement, site) {
            call.line as usize
        } else {
            line
        };
        unanalyzable.push(unanalyzable_finding(rel, report_line, PARSE_GAP));
    }
    for &line in &statement.lock_inventory_unanalyzed {
        let (_, report_line, _) = positions
            .get_or_insert_with(|| Positions::new(call, variant_index))
            .source(line, 1);
        unanalyzable.push(unanalyzable_finding(rel, report_line, ROUTINE_GAP));
    }
}

fn check_embedded_locks(
    check: &VariantContext<'_>,
    statement: &SqlStatementFileFacts,
    positions: &mut Positions<'_>,
    findings: &mut Vec<RuleFinding>,
) {
    for lock in statement
        .lock_inventory
        .iter()
        .filter(|lock| is_lock_category(&lock.category))
    {
        let (source_file, report_line, _) = positions.source(lock.line, lock.column);
        let (_, stmt_source_line, _) =
            positions.source(lock.statement_start_line, lock.statement_start_column);
        let base = check
            .sql_comments
            .status_for_sql_statement(
                lock.statement_start_line,
                lock.statement_start_column,
                lock.statement_end_line,
            )
            .merge(check.call_status);
        let status = merge_local_ts_reason(base, source_file.is_none(), stmt_source_line, check);
        push_reason_finding(check.rel, report_line, &lock.category, status, findings);
    }
}

fn check_embedded_helpers(
    options: &CompiledOptions,
    check: &VariantContext<'_>,
    statement: &SqlStatementFileFacts,
    positions: &mut Positions<'_>,
    findings: &mut Vec<RuleFinding>,
) {
    for (index, function) in statement.function_calls.iter().enumerate() {
        if !matches_sql_helper(options, function) {
            continue;
        }
        let imported = site_is_imported(statement, SqlFactSite::Function(index));
        let report_line = if imported {
            check.call_line
        } else {
            positions.source(function.line, function.column).1
        };
        let base = check
            .sql_comments
            .status_for_sql_line(function.line, function.column)
            .merge(check.call_status);
        let status = merge_local_ts_reason(base, !imported, report_line, check);
        push_reason_finding(check.rel, report_line, "helper", status, findings);
    }
}

fn site_is_imported(statement: &SqlStatementFileFacts, site: SqlFactSite) -> bool {
    statement
        .variant_locations
        .as_ref()
        .and_then(|locations| locations.position(site))
        .is_some_and(|position| position.source_path.is_some())
}

fn merge_local_ts_reason(
    status: ReasonStatus,
    is_local: bool,
    source_line: usize,
    check: &VariantContext<'_>,
) -> ReasonStatus {
    if is_local && source_line != check.call_line {
        if let Some(comments) = check.ts_index {
            return status.merge(comments.status_for_ts_line(source_line));
        }
    }
    status
}

fn variant_sql_text(call: &EmbeddedSqlCall, variant_index: usize) -> &str {
    call.variants
        .get(variant_index)
        .filter(|_| !call.mirrors_single_variant())
        .map(|variant| variant.sql_text.as_str())
        .unwrap_or_else(|| call.sql_text.as_deref().unwrap_or_default())
}
